use dashmap::DashMap;
use log::info;
use rootcause::{
    Report,
    markers::{Dynamic, Mutable, SendSync},
};
use topiary_config::Configuration;
use topiary_core::{Operation, formatter_str};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DocumentFormattingParams, InitializeParams, InitializeResult, MessageType, Position, Range,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    },
};

pub type LspResult<T> = std::result::Result<T, Report<Dynamic, Mutable, SendSync>>;

#[derive(Debug)]
pub struct TopiaryLsp {
    pub client: Client,
    pub config: Configuration,
    pub documents: DashMap<String, String>,
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
                document_formatting_provider: Some(tower_lsp_server::lsp_types::OneOf::Left(true)),
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
        if let Some(change) = params.content_changes.pop() {
            self.documents
                .insert(params.text_document.uri.to_string(), change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri.to_string());
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.clone();
        let uri_str = uri.to_string();

        let source = match self.documents.get(&uri_str) {
            Some(doc) => doc.value().clone(),
            None => return Ok(None),
        };

        let path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        // Detect language from file path, converting error to string to drop non-Send types
        let detect_result = self
            .config
            .detect(&path)
            .map(|lang| lang.name.clone())
            .map_err(|e| e.to_string());

        let language_name = match detect_result {
            Ok(name) => name,
            Err(e_str) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!(
                            "Could not detect language for {}: {}",
                            path.display(),
                            e_str
                        ),
                    )
                    .await;
                return Ok(None);
            }
        };

        // Resolve language (queries, grammar)
        let resolve_result =
            topiary_resolver::resolve_language_by_name(&self.config, &language_name)
                .await
                .map_err(|e| format!("{:?}", e));

        let language = match resolve_result {
            Ok(l) => l,
            Err(e_str) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Failed to resolve language {}: {}", language_name, e_str),
                    )
                    .await;
                return Ok(None);
            }
        };

        // Format the content
        let mut formatted_bytes = Vec::new();
        let operation = Operation::Format {
            skip_idempotence: false,
            tolerate_parsing_errors: true,
        };

        match formatter_str(&source, &mut formatted_bytes, &language, operation, None) {
            Ok(_) => {
                let formatted = match String::from_utf8(formatted_bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        self.client
                            .log_message(
                                MessageType::ERROR,
                                format!("Formatter output invalid UTF-8: {:?}", e),
                            )
                            .await;
                        return Ok(None);
                    }
                };
                // Calculate the end of the document for full replacement
                let lines: Vec<&str> = source.lines().collect();
                let last_line = lines.len().saturating_sub(1) as u32;
                let last_char = lines.last().map(|l| l.len() as u32).unwrap_or(0);

                Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: last_line,
                            character: last_char,
                        },
                    },
                    new_text: formatted,
                }]))
            }
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("Formatting failed: {:?}", e))
                    .await;
                Ok(None)
            }
        }
    }
}

pub async fn run(config: Configuration) -> LspResult<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| TopiaryLsp {
        client,
        config,
        documents: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
