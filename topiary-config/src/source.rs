//! Configuration for Topiary can be sourced from either that which is built-in, or from disk.

use std::{
    env::current_dir,
    ffi::OsString,
    fmt,
    io::Cursor,
    path::{Path, PathBuf},
};

use crate::error::{TopiaryConfigError, TopiaryConfigResult};

/// Sources of Nickel configuration
#[derive(Debug, Clone)]
pub enum Source {
    Builtin,
    Directory(PathBuf),
    File(PathBuf),
}

impl From<Source> for nickel_lang_core::program::Input<Cursor<String>, OsString> {
    fn from(source: Source) -> Self {
        use nickel_lang_core::cache::InputFormat;
        match source {
            Source::Builtin => Self::Source(
                Cursor::new(source.builtin_nickel()),
                "built-in".into(),
                InputFormat::Nickel,
            ),
            Source::Directory(path) => Self::Path(path.into()),
            Source::File(path) => Self::Path(path.into()),
        }
    }
}

impl Source {
    /// Iterate through valid sources of configuration, in priority order (highest to lowest):
    ///
    /// 1. `path`, passed as a CLI argument/environment variable
    /// 2. `.topiary/languages.ncl` (or equivalent)
    /// 3. `~/.config/topiary/languages.ncl`
    /// 4. OS configuration directory (if different from #3)
    /// 5. Built-in configuration: [`Self::builtin_nickel`]
    pub fn config_sources(path: &Option<PathBuf>) -> impl Iterator<Item = (&'static str, Self)> {
        let mut sources = Vec::new();

        if let Some(path) = path {
            let source = if path.is_dir() {
                Self::Directory(path.clone())
            } else {
                Self::File(path.clone())
            };

            sources.push(("CLI", source));
        }

        sources.append(&mut vec![
            ("workspace", workspace_config_dir()),
            #[cfg(target_os = "macos")]
            ("unix-home", unix_home_config_dir()),
            ("OS", os_config_dir()),
            // add built-in config to end
            ("built-in", Self::Builtin),
        ]);

        sources.into_iter()
    }

    /// Return expected query directory associated with the source path
    pub fn queries_dir(&self) -> Option<PathBuf> {
        match self {
            Source::Builtin => None,
            Source::Directory(dir) => Some(dir.join("queries")),
            Source::File(file) => file.parent().map(|d| d.join("queries")),
        }
    }

    // return a config file if able uses `languages.ncl` for directories
    pub fn languages_file(&self) -> Option<PathBuf> {
        match self {
            Source::Builtin => None,
            Source::File(file) => Some(file.clone()),
            Source::Directory(dir) => Some(dir.join("languages.ncl")),
        }
    }

    // return an iterator containing all config sources that have been shown to exist
    fn valid_config_sources(file: &Option<PathBuf>) -> impl Iterator<Item = (&'static str, Self)> {
        Self::config_sources(file).filter_map(|(hint, candidate)| {
            if matches!(candidate, Self::Builtin) {
                return Some((hint, candidate));
            }
            let languages_file = candidate.languages_file().unwrap();
            if !languages_file.exists() {
                log::debug!("configuration file not found: {}.", candidate);
                return None;
            }

            Some((hint, Self::File(languages_file)))
        })
    }
    /// Return all valid configuration sources.
    /// See [`Self::config_sources`].
    pub fn fetch_all(file: &Option<PathBuf>) -> Vec<Self> {
        // We always include the built-in configuration, as a fallback
        log::info!("Adding built-in configuration to merge");
        Self::valid_config_sources(file)
            .inspect(|(hint, candidate)| {
                let Self::File(path) = candidate else { return };
                log::info!(
                    "Adding {hint}-specified configuration to merge: {}",
                    path.display()
                );
            })
            .map(|(_, s)| s)
            .collect()
    }

    /// Checks if a given [`Self`] variant can be found as a path or value
    pub fn languages_exists(&self) -> bool {
        match self {
            Source::Builtin => true,
            Source::File(file) => file.exists(),
            Source::Directory(dir) => dir.join("languages.ncl").exists(),
        }
    }

    /// Return a valid source of configuration with the highest priority.
    /// See [`Self::config_sources`].
    pub fn fetch_one(file: &Option<PathBuf>) -> Self {
        let (hint, source) = Self::valid_config_sources(file)
            .next()
            .expect("built-in should always be present");
        log::info!("Using {hint}-specified configuration: {source}");
        source
    }

    #[allow(clippy::result_large_err)]
    pub fn read(&self) -> TopiaryConfigResult<Vec<u8>> {
        match self {
            Self::Builtin => Ok(self.builtin_nickel().into_bytes()),

            Self::Directory(dir) => read_to_string(&dir.join("languages.ncl")),
            Self::File(path) => read_to_string(path),
        }
    }

    fn builtin_nickel(&self) -> String {
        include_str!("../languages.ncl").to_string()
    }
}

fn read_to_string(path: &Path) -> TopiaryConfigResult<Vec<u8>> {
    std::fs::read_to_string(path)
        .map_err(TopiaryConfigError::Io)
        .map(|s| s.into_bytes())
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin => write!(f, "<built-in>"),

            Self::File(path) | Self::Directory(path) => {
                // If the configuration is provided through a file, then we know by this point that
                // it must exist and so the call to `canonicalize` will succeed. However, special
                // cases -- such as process substitution, which creates a temporary FIFO -- may
                // fail if the shell has cleaned things up from under us; in which case, we
                // fallback to the original `path`.
                let config = path.canonicalize().unwrap_or(path.clone());
                write!(f, "{}", config.display())
            }
        }
    }
}

/// Find the OS-specific configuration directory
/// Directory is not guaranteed to exist.
fn os_config_dir() -> Source {
    Source::Directory(crate::project_dirs().config_dir().to_path_buf())
}

/// Ascend the directory hierarchy, starting from the current working directory, in search of the
/// nearest `.topiary` configuration directory.
/// Directory is not guaranteed to exist.
fn workspace_config_dir() -> Source {
    let pwd = current_dir().expect("Could not get current working directory");
    let dir = pwd
        .ancestors()
        .map(|path| path.join(".topiary"))
        .find(|path| path.exists())
        .unwrap_or_else(|| pwd.join(".topiary"));

    Source::Directory(dir)
}

/// Certain platforms have alternate config directories (macOS)
/// polyfill for linux-like `os_config_dir()`
/// https://docs.rs/directories/latest/src/directories/lib.rs.html#38-43
/// Directory is not guaranteed to exist.
#[cfg(target_os = "macos")]
fn unix_home_config_dir() -> Source {
    let dir = std::env::home_dir()
        .unwrap_or_default()
        .join(".config/topiary");

    Source::Directory(dir)
}
