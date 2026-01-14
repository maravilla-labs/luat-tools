// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::Document;
use crate::handlers;

pub struct LuatLanguageServer {
    client: Client,
    documents: DashMap<Url, Document>,
}

impl LuatLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Get a document by URI (for external access)
    #[allow(dead_code)] // Useful for future features like workspace-wide operations
    pub fn get_document(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, Document>> {
        self.documents.get(uri)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LuatLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("Initializing luat-lsp");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        "#".to_string(),
                        ":".to_string(),
                        "@".to_string(),
                        "<".to_string(),
                        ".".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "luat-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("luat-lsp initialized");
        self.client
            .log_message(MessageType::INFO, "Luat language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down luat-lsp");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        tracing::debug!("Document opened: {}", uri);

        let document = Document::new(uri.clone(), text);
        self.documents.insert(uri.clone(), document);

        // Publish initial diagnostics
        let diagnostics = if let Some(doc) = self.documents.get(&uri) {
            handlers::diagnostics::compute_diagnostics(&doc)
        } else {
            vec![]
        };

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(mut doc) = self.documents.get_mut(&uri) {
            for change in params.content_changes {
                doc.apply_change(&change);
            }

            // Re-publish diagnostics
            let diagnostics = handlers::diagnostics::compute_diagnostics(&doc);
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document closed: {}", uri);
        self.documents.remove(&uri);

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(uri) {
            let completions = handlers::completion::get_completions(&doc, position);
            Ok(Some(CompletionResponse::List(CompletionList {
                is_incomplete: false,
                items: completions,
            })))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            Ok(handlers::hover::get_hover(&doc, position))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            Ok(handlers::definition::get_definition(&doc, position))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            Ok(handlers::symbols::get_document_symbols(&doc))
        } else {
            Ok(None)
        }
    }
}
