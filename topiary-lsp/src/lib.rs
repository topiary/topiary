use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use dashmap::DashMap;
use log::info;
use rootcause::{
    Report,
    markers::{Dynamic, Mutable, SendSync},
};
use topiary_config::{Configuration, error::TopiaryConfigError, language::LocalRepos};
use topiary_core::{FormatterError, FormatterResult, Language, Operation, formatter_str};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DocumentFormattingParams, InitializeParams, InitializeResult, MessageType, OneOf, Position,
        Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    },
};

pub type LspResult<T> = std::result::Result<T, Report<Dynamic, Mutable, SendSync>>;

/// Resolves and caches language definitions for the lifetime of the server.
///
/// Resolving a language means fetching (and possibly building) its grammar and
/// parsing its queries, which is far too expensive to repeat on every
/// formatting request. Definitions are cached by name; changes to query files
/// or the configuration are picked up when the server restarts.
#[derive(Debug)]
struct Languages {
    config: Configuration,
    repos: LocalRepos,
    cache: Mutex<HashMap<String, Arc<Language>>>,
    /// Languages excluded via `--skip-language`.
    skip_languages: HashSet<String>,
}

impl Languages {
    fn get(&self, name: &str) -> topiary_resolver::ResolverResult<Arc<Language>> {
        // Hold the lock while resolving, so concurrent requests for the same
        // language don't both fetch the grammar.
        let mut cache = self.cache.lock().expect("language cache mutex poisoned");
        if let Some(language) = cache.get(name) {
            return Ok(language.clone());
        }
        let language = Arc::new(topiary_resolver::resolve_language_by_name_with(
            &self.config,
            name,
            &self.repos,
        )?);
        cache.insert(name.to_owned(), language.clone());
        Ok(language)
    }

    /// Resolver for injected languages, mirroring the CLI: unknown or skipped
    /// languages are left unformatted, other failures are errors.
    fn resolve_injected(&self, name: &str) -> FormatterResult<Option<Arc<Language>>> {
        if matches!(
            self.config.get_language_cfg(name),
            Err(TopiaryConfigError::UnknownLanguage(_))
        ) {
            return Ok(None);
        }
        if self.skip_languages.contains(name) {
            log::debug!("Skipping injected language: {name}");
            return Ok(None);
        }
        self.get(name).map(Some).map_err(|report| {
            report.context(FormatterError::InjectionLanguageResolution {
                language: name.to_owned(),
            })
        })
    }

    /// Format `source` as a document at `path`. Blocking: may fetch grammars.
    fn format(&self, path: &std::path::Path, source: &str) -> std::result::Result<String, String> {
        let name = self
            .config
            .detect(path)
            .map(|language| language.name.clone())
            .map_err(|e| format!("Could not detect language for {}: {e}", path.display()))?;

        if self.skip_languages.contains(&name) {
            return Err(format!(
                "Not formatting {}: language {name} is excluded via --skip-language",
                path.display()
            ));
        }

        let language = self
            .get(&name)
            .map_err(|e| format!("Failed to resolve language {name}: {e:?}"))?;

        let mut output = Vec::new();
        formatter_str(
            source,
            &mut output,
            &language,
            Operation::Format {
                skip_idempotence: false,
                // Deliberately strict: formatting around parse errors can
                // mangle a buffer that is mid-edit. Refuse instead.
                tolerate_parsing_errors: false,
                skip_stage: None,
            },
            Some(&|injected| self.resolve_injected(injected)),
        )
        .map_err(|e| format!("Formatting failed: {e:?}"))?;

        String::from_utf8(output).map_err(|e| format!("Formatter output invalid UTF-8: {e}"))
    }
}

#[derive(Debug)]
pub struct TopiaryLsp {
    client: Client,
    languages: Arc<Languages>,
    documents: DashMap<String, String>,
}

#[tower_lsp_server::async_trait]
impl LanguageServer for TopiaryLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing topiary-lsp");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: tower_lsp_server::lsp_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "topiary-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(
            params.text_document.uri.to_string(),
            params.text_document.text,
        );
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        // With full sync, the last change holds the whole document.
        if let Some(change) = params.content_changes.pop() {
            self.documents
                .insert(params.text_document.uri.to_string(), change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri.to_string());
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        let Some(source) = self
            .documents
            .get(&uri.to_string())
            .map(|d| d.value().clone())
        else {
            return Ok(None);
        };

        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };

        // Formatting is CPU-bound and resolving a language may fetch and build
        // a grammar, so keep it off the async executor.
        let languages = self.languages.clone();
        let result = {
            let source = source.clone();
            tokio::task::spawn_blocking(move || languages.format(&path, &source)).await
        };

        match result {
            Ok(Ok(formatted)) => Ok(Some(full_document_edit(&source, formatted))),
            Ok(Err(message)) => {
                self.client.log_message(MessageType::ERROR, message).await;
                Ok(None)
            }
            Err(join_error) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Formatting task failed: {join_error}"),
                    )
                    .await;
                Ok(None)
            }
        }
    }
}

/// Edits turning `source` into `formatted`: none if unchanged, otherwise a
/// single edit replacing the whole document.
fn full_document_edit(source: &str, formatted: String) -> Vec<TextEdit> {
    if source == formatted {
        return Vec::new();
    }
    vec![TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end: end_position(source),
        },
        new_text: formatted,
    }]
}

/// The LSP position just past the last character of `text`.
///
/// Lines are separated by `\n`, `\r\n` or `\r`, and characters are counted in
/// UTF-16 code units, the encoding LSP uses by default.
fn end_position(text: &str) -> Position {
    let mut line = 0;
    let mut character = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                chars.next_if_eq(&'\n');
                line += 1;
                character = 0;
            }
            '\n' => {
                line += 1;
                character = 0;
            }
            c => character += c.len_utf16() as u32,
        }
    }
    Position::new(line, character)
}

pub async fn run(
    config: Configuration,
    skip_languages: impl IntoIterator<Item = String>,
) -> LspResult<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let languages = Arc::new(Languages {
        config,
        repos: LocalRepos::new(),
        cache: Mutex::new(HashMap::new()),
        skip_languages: skip_languages.into_iter().collect(),
    });

    let (service, socket) = LspService::new(|client| TopiaryLsp {
        client,
        languages,
        documents: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_position_empty() {
        assert_eq!(end_position(""), Position::new(0, 0));
    }

    #[test]
    fn end_position_counts_trailing_newline() {
        assert_eq!(end_position("a\n"), Position::new(1, 0));
        assert_eq!(end_position("a\nbc"), Position::new(1, 2));
    }

    #[test]
    fn end_position_line_terminators() {
        assert_eq!(end_position("a\r\nb"), Position::new(1, 1));
        assert_eq!(end_position("a\rb"), Position::new(1, 1));
        assert_eq!(end_position("a\r\n"), Position::new(1, 0));
    }

    #[test]
    fn end_position_utf16() {
        // 'é' is one UTF-16 unit (two UTF-8 bytes); '😀' is two (four bytes).
        assert_eq!(end_position("é😀"), Position::new(0, 3));
    }

    #[test]
    fn unchanged_document_needs_no_edit() {
        assert!(full_document_edit("a\n", "a\n".to_owned()).is_empty());
    }

    #[test]
    fn edit_replaces_whole_document() {
        let edits = full_document_edit("a  \n", "a\n".to_owned());
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.end, Position::new(1, 0));
    }
}
