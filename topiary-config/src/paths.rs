//! Resolution of relative filesystem paths written in a Nickel configuration.
//!
//! A configuration may point Topiary at files on disk:
//!
//! ```nickel
//! { languages.nickel.queries.formatting.source.path = "./formatting.scm" }
//! ```
//!
//! Handed to `std::fs` as-is, such a path resolves against the process's working
//! directory, so the configuration only works from wherever it happened to be
//! written for. Instead we anchor it at the directory of the `.ncl` file that
//! *defined* it, which is also how Nickel resolves its own `import`s.
//!
//! We do this on the *evaluated* configuration: by then Nickel has performed all
//! merging, contract application and `| default` overriding, so every surviving
//! string literal is the one that actually won, and its [`PosIdx`] points at the
//! file it was written in.

use std::path::{Path, PathBuf};

use nickel_lang_core::{
    eval::value::{NickelValue, RecordData, ValueContentRefMut},
    files::Files,
    identifier::Ident,
    position::{PosIdx, PosTable},
};

/// The two paths in a language's configuration that name a file on the local
/// filesystem. Both live in a record called `source`; see [`PathResolver::resolve_source`]
/// for the caveat about the sibling `git` field.
const GRAMMAR: &str = "grammar";
const QUERIES: &str = "queries";
const SOURCE: &str = "source";
const PATH: &str = "path";
const GIT: &str = "git";

/// Rewrites relative `path` values in an evaluated configuration so that they are
/// anchored at the `.ncl` file that defined them, rather than at the working directory.
pub(crate) struct PathResolver<'a> {
    table: &'a PosTable,
    files: Files,
}

impl<'a> PathResolver<'a> {
    /// `files` is cloned out of the program once, rather than per lookup:
    /// `Program::files` hands back an owned copy of the whole registry.
    pub(crate) fn new(table: &'a PosTable, files: Files) -> Self {
        Self { table, files }
    }

    /// Walk `languages.<lang>` for the records that name a local file, and resolve them.
    pub(crate) fn resolve(&self, config: &mut NickelValue) {
        let Some(languages) = as_record_mut(config).and_then(|c| field_mut(c, "languages")) else {
            return;
        };
        let Some(languages) = as_record_mut(languages) else {
            return;
        };

        for (_, language) in languages.fields.iter_mut() {
            let Some(language) = language.value.as_mut().and_then(as_record_mut) else {
                continue;
            };

            if let Some(grammar) = field_mut(language, GRAMMAR)
                && let Some(grammar) = as_record_mut(grammar)
                && let Some(source) = field_mut(grammar, SOURCE)
            {
                self.resolve_source(source);
            }

            let Some(queries) = field_mut(language, QUERIES).and_then(as_record_mut) else {
                continue;
            };

            for (_, query) in queries.fields.iter_mut() {
                if let Some(query) = query.value.as_mut().and_then(as_record_mut)
                    && let Some(source) = field_mut(query, SOURCE)
                {
                    self.resolve_source(source);
                }
            }
        }
    }

    /// A `source` is either `{ path }` or `{ git, path }`. Only the former names a path on
    /// the local filesystem: when `git` is present, `path` names a file *inside* the
    /// checkout Topiary fetches, and must be left alone.
    fn resolve_source(&self, source: &mut NickelValue) {
        let Some(source) = as_record_mut(source) else {
            return;
        };
        if source.fields.contains_key(&Ident::new(GIT)) {
            return;
        }

        let Some(path) = field_mut(source, PATH) else {
            return;
        };
        let Some(relative) = path.as_string().map(|s| PathBuf::from(s.as_str())) else {
            return;
        };
        // `is_relative` is the wrong test on Windows, where a rooted but drive-less path
        // such as `\queries\formatting.scm` is "relative" -- to the current drive -- yet
        // already anchored. Joining it onto the configuration's directory would silently
        // re-root it onto that directory's drive. Only a path with no root needs a base.
        if relative.has_root() {
            return;
        }

        let pos_idx = path.pos_idx();
        let Some(dir) = self.defining_dir(pos_idx) else {
            return;
        };

        // Collecting the components drops the `.` of a `"./foo"`, which would otherwise
        // survive into error messages and `topiary cfg` output as `<dir>/./foo`.
        let resolved: PathBuf = dir.join(relative).components().collect();
        log::debug!(
            "resolved {} to {}",
            path.as_string().expect("checked just above"),
            resolved.display()
        );
        *path = NickelValue::string(resolved.to_string_lossy().into_owned(), pos_idx);
    }

    /// The directory holding the `.ncl` file a value was written in.
    ///
    /// `None` when the value carries no position, or when its source is not a file on
    /// disk. The latter covers the built-in configuration: it is registered from an
    /// in-memory buffer under the name `built-in`, which -- unlike a path registered with
    /// `add_file` -- Nickel does not normalise into an absolute path, and which names no
    /// real file. Its paths are therefore left exactly as written.
    fn defining_dir(&self, pos_idx: PosIdx) -> Option<PathBuf> {
        let span = self.table.get(pos_idx).into_opt()?;
        let file = Path::new(self.files.name(span.src_id));

        file.is_file()
            .then(|| file.parent())
            .flatten()
            .map(Path::to_path_buf)
    }
}

/// A mutable view of `value` as a record, or `None` if it is not one.
///
/// `content_make_mut` rather than `content_mut`: the former is the copy-on-write
/// accessor, and so succeeds even when the value block is shared.
fn as_record_mut(value: &mut NickelValue) -> Option<&mut RecordData> {
    match value.content_make_mut() {
        ValueContentRefMut::Record(record) => record.into_opt(),
        _ => None,
    }
}

/// The value of `record.<name>`, or `None` when the field is absent or has no value
/// (an `optional` field that was never defined).
fn field_mut<'a>(record: &'a mut RecordData, name: &str) -> Option<&'a mut NickelValue> {
    record.fields.get_mut(&Ident::new(name))?.value.as_mut()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::fs;

    use crate::{
        Configuration,
        language::{GrammarSource, QuerySource},
    };

    use super::*;

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().expect("path has a parent")).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn fetch(config_file: &Path) -> Configuration {
        Configuration::fetch(false, Some(config_file)).unwrap().0
    }

    fn query_source(config: &Configuration, language: &str, query: &str) -> QuerySource {
        config
            .get_language_cfg(language)
            .unwrap()
            .config_query(query)
            .unwrap()
            .source
            .clone()
    }

    #[test]
    fn relative_query_path_is_anchored_at_the_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config_file = tmp.path().join("languages.ncl");
        write(
            &config_file,
            r#"{ languages.markdown.queries.formatting.source.path = "./queries/markdown/formatting.scm" }"#,
        );

        let config = fetch(&config_file);

        assert_eq!(
            query_source(&config, "markdown", "formatting").path,
            tmp.path().join("queries/markdown/formatting.scm")
        );
    }

    /// A path that is already rooted is left as written. On Windows this covers the
    /// drive-less `/somewhere/else` form, which is rooted without being absolute.
    #[test]
    fn rooted_query_path_is_left_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let config_file = tmp.path().join("languages.ncl");
        write(
            &config_file,
            r#"{ languages.markdown.queries.formatting.source.path = "/somewhere/else/formatting.scm" }"#,
        );

        let config = fetch(&config_file);

        assert_eq!(
            query_source(&config, "markdown", "formatting").path,
            Path::new("/somewhere/else/formatting.scm")
        );
    }

    #[test]
    fn git_backed_query_path_is_left_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let config_file = tmp.path().join("languages.ncl");
        write(
            &config_file,
            r#"{
              languages.markdown.queries.formatting.source = {
                git = { git = "https://example.invalid/repo.git", rev = "deadbeef" },
                path = "queries/markdown/formatting.scm",
              },
            }"#,
        );

        let source = query_source(&fetch(&config_file), "markdown", "formatting");

        // With `git`, the path names a file inside the checkout, not on the local disk
        assert!(source.git.is_some());
        assert_eq!(
            source.path,
            Path::new("queries/markdown/formatting.scm"),
            "a git-backed path must stay relative to the checkout root"
        );
    }

    #[test]
    fn relative_grammar_path_is_anchored_at_the_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config_file = tmp.path().join("languages.ncl");
        write(
            &config_file,
            r#"{ languages.markdown.grammar.source.path = "./tree-sitter-markdown.so" }"#,
        );

        let config = fetch(&config_file);
        let grammar = &config.get_language_cfg("markdown").unwrap().config.grammar;

        assert_eq!(
            grammar.source,
            GrammarSource::Path(tmp.path().join("tree-sitter-markdown.so"))
        );
    }

    /// Each path is anchored at the file it was written in, not at the entry point: the
    /// position of a value survives Nickel's import resolution and merging.
    #[test]
    fn each_path_is_anchored_at_its_own_file() {
        let tmp = tempfile::tempdir().unwrap();

        let imported = tmp.path().join("imported/languages.ncl");
        write(
            &imported,
            r#"{ languages.markdown.queries.formatting.source.path = "./formatting.scm" }"#,
        );

        let config_file = tmp.path().join("entrypoint/languages.ncl");
        write(
            &config_file,
            r#"(import "../imported/languages.ncl")
               & { languages.rust.queries.formatting.source.path = "./formatting.scm" }"#,
        );

        let config = fetch(&config_file);

        assert_eq!(
            query_source(&config, "markdown", "formatting").path,
            tmp.path().join("imported/formatting.scm")
        );
        assert_eq!(
            query_source(&config, "rust", "formatting").path,
            tmp.path().join("entrypoint/formatting.scm")
        );
    }

    /// The built-in configuration is registered from an in-memory buffer, so it has no
    /// directory to anchor against and must come through untouched.
    ///
    /// Asserted against the unresolved evaluation rather than against a known grammar
    /// source, because the Nix build swaps every built-in `grammar.source.git` for a
    /// `/nix/store` path (see `nix/utils/prefetchLanguages.nix`).
    #[test]
    fn built_in_configuration_is_left_alone() {
        let mut program = crate::Program::build_with_sources(&[crate::Source::Builtin]).unwrap();

        let unresolved = program.eval_full_for_export().unwrap();
        let resolved = program.eval_config().unwrap();

        assert_eq!(unresolved.to_string(), resolved.to_string());
    }
}
