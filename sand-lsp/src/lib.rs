//! Language server for the Sand datapack language.
//!
//! The server speaks the Language Server Protocol over stdio and answers every
//! request from a [`sand_lang::Analysis`] of the package that owns the file.
//! A package is found by walking up from a file to the nearest `pack.sand`, so
//! opening any module file analyses the whole pack, including unsaved buffers.
//!
//! Supported features:
//!
//! - diagnostics from parsing, name resolution, type checking, execution
//!   context inference, and Sand's portable compiler backend
//! - hover with signatures, inferred execution context, identities, and docs
//! - go to definition, find references, rename with prepare
//! - context-aware completion for items, `::` paths, State fields after `.`,
//!   enum variants, attributes, and snippets
//! - hierarchical document symbols and inferred-context inlay hints
//!
//! Run it with `sand lsp`; editors launch that command as a stdio server.

use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types as lsp;
use sand_lang::ide;
use sand_lang::{Analysis, FileId, LineIndex, OverlayFiles, Severity, ast::Span};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Run the server over stdin/stdout until the client asks it to exit.
pub fn run_stdio() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    run(connection)?;
    io_threads.join()?;
    Ok(())
}

/// Run the server over an established connection; used by tests.
pub fn run(connection: Connection) -> Result<()> {
    let capabilities = serde_json::to_value(server_capabilities())?;
    connection.initialize(capabilities)?;
    let mut server = Server::default();
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                let response = server.request(request);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(notification) => {
                for out in server.notification(notification) {
                    connection.sender.send(Message::Notification(out))?;
                }
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

/// The capabilities advertised during initialization.
pub fn server_capabilities() -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
        position_encoding: Some(lsp::PositionEncodingKind::UTF16),
        text_document_sync: Some(lsp::TextDocumentSyncCapability::Options(
            lsp::TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(lsp::TextDocumentSyncKind::INCREMENTAL),
                save: Some(lsp::TextDocumentSyncSaveOptions::Supported(true)),
                ..Default::default()
            },
        )),
        hover_provider: Some(lsp::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp::OneOf::Left(true)),
        references_provider: Some(lsp::OneOf::Left(true)),
        rename_provider: Some(lsp::OneOf::Right(lsp::RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: Default::default(),
        })),
        completion_provider: Some(lsp::CompletionOptions {
            trigger_characters: Some(vec![".".into(), ":".into(), "[".into()]),
            ..Default::default()
        }),
        signature_help_provider: Some(lsp::SignatureHelpOptions {
            trigger_characters: Some(vec!["(".into(), ",".into()]),
            retrigger_characters: Some(vec![" ".into(), "=".into()]),
            work_done_progress_options: Default::default(),
        }),
        document_symbol_provider: Some(lsp::OneOf::Left(true)),
        inlay_hint_provider: Some(lsp::OneOf::Left(true)),
        ..Default::default()
    }
}

/// Server state: open buffers and the diagnostics last published per package.
#[derive(Default)]
pub struct Server {
    open: HashMap<PathBuf, String>,
    published: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl Server {
    fn files(&self) -> OverlayFiles {
        let mut files = OverlayFiles::with_disk();
        files.overlay = self.open.clone();
        files
    }

    /// Analyse the package that owns `path`.
    pub fn analysis_for(&self, path: &Path) -> (Analysis, Option<FileId>) {
        let files = self.files();
        let root = sand_lang::find_root(path, &files);
        let analysis = sand_lang::analyze_path(&root, &files);
        if let Some(file) = analysis.package.file_id(path) {
            return (analysis, Some(file));
        }
        // Not reachable from the package root (no `mod` declares it yet):
        // analyse the file on its own so it still gets diagnostics and hover.
        let analysis = sand_lang::analyze_path(path, &files);
        let file = analysis.package.file_id(path);
        (analysis, file)
    }

    fn notification(&mut self, notification: Notification) -> Vec<Notification> {
        use lsp::notification::Notification as _;
        let path = match notification.method.as_str() {
            lsp::notification::DidOpenTextDocument::METHOD => {
                let Ok(params) =
                    serde_json::from_value::<lsp::DidOpenTextDocumentParams>(notification.params)
                else {
                    return Vec::new();
                };
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return Vec::new();
                };
                self.open.insert(path.clone(), params.text_document.text);
                path
            }
            lsp::notification::DidChangeTextDocument::METHOD => {
                let Ok(params) =
                    serde_json::from_value::<lsp::DidChangeTextDocumentParams>(notification.params)
                else {
                    return Vec::new();
                };
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return Vec::new();
                };
                let text = self.open.entry(path.clone()).or_default();
                for change in params.content_changes {
                    apply_change(text, change);
                }
                path
            }
            lsp::notification::DidSaveTextDocument::METHOD => {
                let Ok(params) =
                    serde_json::from_value::<lsp::DidSaveTextDocumentParams>(notification.params)
                else {
                    return Vec::new();
                };
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return Vec::new();
                };
                path
            }
            lsp::notification::DidCloseTextDocument::METHOD => {
                let Ok(params) =
                    serde_json::from_value::<lsp::DidCloseTextDocumentParams>(notification.params)
                else {
                    return Vec::new();
                };
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return Vec::new();
                };
                self.open.remove(&path);
                path
            }
            _ => return Vec::new(),
        };
        self.diagnose(&path)
    }

    /// Analyse the package owning `path` and publish diagnostics for every
    /// file in it, clearing files that no longer have any.
    pub fn diagnose(&mut self, path: &Path) -> Vec<Notification> {
        let (analysis, _) = self.analysis_for(path);
        let mut diagnostics = analysis.diagnostics.clone();
        if !analysis.has_errors()
            && let Err(errors) = analysis.compile()
        {
            diagnostics.extend(errors);
        }
        let mut by_file: HashMap<PathBuf, Vec<lsp::Diagnostic>> = HashMap::new();
        for file in &analysis.package.files {
            by_file.entry(file.path.clone()).or_default();
        }
        for diagnostic in &diagnostics {
            let source = &analysis.package.files[diagnostic.file.0];
            by_file
                .entry(source.path.clone())
                .or_default()
                .push(to_lsp_diagnostic(&analysis, diagnostic));
        }
        let root = analysis.package.root.clone();
        let previous = self.published.remove(&root).unwrap_or_default();
        for stale in previous {
            by_file.entry(stale).or_default();
        }
        self.published
            .insert(root, by_file.keys().cloned().collect());
        let mut paths: Vec<_> = by_file.into_iter().collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));
        paths
            .into_iter()
            .filter_map(|(path, diagnostics)| {
                let params = lsp::PublishDiagnosticsParams {
                    uri: path_to_uri(&path)?,
                    diagnostics,
                    version: None,
                };
                Some(Notification::new(
                    <lsp::notification::PublishDiagnostics as lsp::notification::Notification>::METHOD
                        .to_string(),
                    params,
                ))
            })
            .collect()
    }

    fn request(&mut self, request: Request) -> Response {
        let id = request.id.clone();
        match self.dispatch(request) {
            Ok(value) => Response::new_ok(id, value),
            Err(message) => Response::new_err(id, ErrorCode::RequestFailed as i32, message),
        }
    }

    fn dispatch(
        &mut self,
        request: lsp_server::Request,
    ) -> std::result::Result<serde_json::Value, String> {
        use lsp::request::*;
        fn parse<R: lsp::request::Request>(
            request: lsp_server::Request,
        ) -> std::result::Result<R::Params, String> {
            serde_json::from_value(request.params).map_err(|e| e.to_string())
        }
        let json = |value: serde_json::Value| Ok(value);
        match request.method.as_str() {
            HoverRequest::METHOD => {
                let params = parse::<HoverRequest>(request)?;
                let at = self.locate(&params.text_document_position_params)?;
                let Some((analysis, file, offset)) = at else {
                    return json(serde_json::Value::Null);
                };
                let hover = ide::hover(&analysis, file, offset).map(|hover| lsp::Hover {
                    contents: lsp::HoverContents::Markup(lsp::MarkupContent {
                        kind: lsp::MarkupKind::Markdown,
                        value: hover.markdown,
                    }),
                    range: Some(range(&analysis, file, hover.span)),
                });
                json(serde_json::to_value(hover).unwrap_or_default())
            }
            GotoDefinition::METHOD => {
                let params = parse::<GotoDefinition>(request)?;
                let Some((analysis, file, offset)) =
                    self.locate(&params.text_document_position_params)?
                else {
                    return json(serde_json::Value::Null);
                };
                let location = ide::definition(&analysis, file, offset)
                    .and_then(|(file, span)| location(&analysis, file, span))
                    .map(lsp::GotoDefinitionResponse::Scalar);
                json(serde_json::to_value(location).unwrap_or_default())
            }
            References::METHOD => {
                let params = parse::<References>(request)?;
                let Some((analysis, file, offset)) = self.locate(&params.text_document_position)?
                else {
                    return json(serde_json::Value::Null);
                };
                let locations: Vec<lsp::Location> =
                    ide::references(&analysis, file, offset, params.context.include_declaration)
                        .into_iter()
                        .filter_map(|(file, span)| location(&analysis, file, span))
                        .collect();
                json(serde_json::to_value(locations).unwrap_or_default())
            }
            PrepareRenameRequest::METHOD => {
                let params = parse::<PrepareRenameRequest>(request)?;
                let Some((analysis, file, offset)) = self.locate(&params)? else {
                    return json(serde_json::Value::Null);
                };
                let span = ide::prepare_rename(&analysis, file, offset)?;
                json(
                    serde_json::to_value(lsp::PrepareRenameResponse::Range(range(
                        &analysis, file, span,
                    )))
                    .unwrap_or_default(),
                )
            }
            Rename::METHOD => {
                let params = parse::<Rename>(request)?;
                let Some((analysis, file, offset)) = self.locate(&params.text_document_position)?
                else {
                    return json(serde_json::Value::Null);
                };
                let edits = ide::rename(&analysis, file, offset, &params.new_name)?;
                // `WorkspaceEdit` requires `Uri` keys; their hashed text is never mutated.
                #[allow(clippy::mutable_key_type)]
                let mut changes: HashMap<lsp::Uri, Vec<lsp::TextEdit>> = HashMap::new();
                for edit in edits {
                    let Some(uri) = path_to_uri(&analysis.package.files[edit.file.0].path) else {
                        continue;
                    };
                    changes.entry(uri).or_default().push(lsp::TextEdit {
                        range: range(&analysis, edit.file, edit.span),
                        new_text: edit.text,
                    });
                }
                json(
                    serde_json::to_value(lsp::WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    })
                    .unwrap_or_default(),
                )
            }
            Completion::METHOD => {
                let params = parse::<Completion>(request)?;
                let Some((analysis, file, offset)) = self.locate(&params.text_document_position)?
                else {
                    return json(serde_json::Value::Null);
                };
                let items: Vec<lsp::CompletionItem> = ide::completions(&analysis, file, offset)
                    .into_iter()
                    .map(completion_item)
                    .collect();
                json(
                    serde_json::to_value(lsp::CompletionResponse::Array(items)).unwrap_or_default(),
                )
            }
            SignatureHelpRequest::METHOD => {
                let params = parse::<SignatureHelpRequest>(request)?;
                let Some((analysis, file, offset)) =
                    self.locate(&params.text_document_position_params)?
                else {
                    return json(serde_json::Value::Null);
                };
                let help = ide::signature_help(&analysis, file, offset).map(|help| {
                    let utf16 = |byte: usize| help.label[..byte].encode_utf16().count() as u32;
                    lsp::SignatureHelp {
                        signatures: vec![lsp::SignatureInformation {
                            label: help.label.clone(),
                            documentation: help.documentation.clone().map(|value| {
                                lsp::Documentation::MarkupContent(lsp::MarkupContent {
                                    kind: lsp::MarkupKind::Markdown,
                                    value,
                                })
                            }),
                            parameters: Some(
                                help.params
                                    .iter()
                                    .map(|(start, end, _)| lsp::ParameterInformation {
                                        label: lsp::ParameterLabel::LabelOffsets([
                                            utf16(*start),
                                            utf16(*end),
                                        ]),
                                        documentation: None,
                                    })
                                    .collect(),
                            ),
                            active_parameter: Some(help.active as u32),
                        }],
                        active_signature: Some(0),
                        active_parameter: Some(help.active as u32),
                    }
                });
                json(serde_json::to_value(help).unwrap_or_default())
            }
            DocumentSymbolRequest::METHOD => {
                let params = parse::<DocumentSymbolRequest>(request)?;
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return json(serde_json::Value::Null);
                };
                let (analysis, file) = self.analysis_for(&path);
                let Some(file) = file else {
                    return json(serde_json::Value::Null);
                };
                let symbols: Vec<lsp::DocumentSymbol> = ide::document_symbols(&analysis, file)
                    .into_iter()
                    .map(|symbol| document_symbol(&analysis, file, symbol))
                    .collect();
                json(
                    serde_json::to_value(lsp::DocumentSymbolResponse::Nested(symbols))
                        .unwrap_or_default(),
                )
            }
            InlayHintRequest::METHOD => {
                let params = parse::<InlayHintRequest>(request)?;
                let Some(path) = uri_to_path(&params.text_document.uri) else {
                    return json(serde_json::Value::Null);
                };
                let (analysis, file) = self.analysis_for(&path);
                let Some(file) = file else {
                    return json(serde_json::Value::Null);
                };
                let index = &analysis.package.files[file.0].lines;
                let hints: Vec<lsp::InlayHint> = ide::inlay_hints(&analysis, file)
                    .into_iter()
                    .map(|hint| lsp::InlayHint {
                        position: position(index, hint.offset),
                        label: lsp::InlayHintLabel::String(hint.label),
                        kind: Some(lsp::InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: hint.tooltip.map(lsp::InlayHintTooltip::String),
                        padding_left: Some(false),
                        padding_right: Some(true),
                        data: None,
                    })
                    .collect();
                json(serde_json::to_value(hints).unwrap_or_default())
            }
            other => Err(format!("unsupported request `{other}`")),
        }
    }

    fn locate(
        &self,
        params: &lsp::TextDocumentPositionParams,
    ) -> std::result::Result<Option<(Analysis, FileId, usize)>, String> {
        let Some(path) = uri_to_path(&params.text_document.uri) else {
            return Ok(None);
        };
        let (analysis, file) = self.analysis_for(&path);
        let Some(file) = file else {
            return Ok(None);
        };
        let offset =
            analysis.package.files[file.0]
                .lines
                .offset_utf16(sand_lang::source::LineCol {
                    line: params.position.line,
                    col: params.position.character,
                });
        Ok(Some((analysis, file, offset)))
    }
}

fn apply_change(text: &mut String, change: lsp::TextDocumentContentChangeEvent) {
    match change.range {
        None => *text = change.text,
        Some(r) => {
            let index = LineIndex::new(text);
            let start = index.offset_utf16(sand_lang::source::LineCol {
                line: r.start.line,
                col: r.start.character,
            });
            let end = index.offset_utf16(sand_lang::source::LineCol {
                line: r.end.line,
                col: r.end.character,
            });
            text.replace_range(start..end.max(start), &change.text);
        }
    }
}

fn position(index: &LineIndex, offset: usize) -> lsp::Position {
    let pos = index.utf16(offset);
    lsp::Position {
        line: pos.line,
        character: pos.col,
    }
}

fn range(analysis: &Analysis, file: FileId, span: Span) -> lsp::Range {
    let index = &analysis.package.files[file.0].lines;
    lsp::Range {
        start: position(index, span.start),
        end: position(index, span.end),
    }
}

fn location(analysis: &Analysis, file: FileId, span: Span) -> Option<lsp::Location> {
    Some(lsp::Location {
        uri: path_to_uri(&analysis.package.files[file.0].path)?,
        range: range(analysis, file, span),
    })
}

fn to_lsp_diagnostic(analysis: &Analysis, diagnostic: &sand_lang::Diagnostic) -> lsp::Diagnostic {
    let mut message = diagnostic.message.clone();
    if let Some(help) = &diagnostic.help {
        message.push_str("\nhelp: ");
        message.push_str(help);
    }
    let related = diagnostic
        .notes
        .iter()
        .filter_map(|note| {
            Some(lsp::DiagnosticRelatedInformation {
                location: location(analysis, note.file, note.span)?,
                message: note.message.clone(),
            })
        })
        .collect::<Vec<_>>();
    lsp::Diagnostic {
        range: range(analysis, diagnostic.file, diagnostic.span),
        severity: Some(match diagnostic.severity {
            Severity::Error => lsp::DiagnosticSeverity::ERROR,
            Severity::Warning => lsp::DiagnosticSeverity::WARNING,
        }),
        code: Some(lsp::NumberOrString::String(diagnostic.code.clone())),
        source: Some("sand".into()),
        message,
        related_information: (!related.is_empty()).then_some(related),
        ..Default::default()
    }
}

fn completion_item(completion: ide::Completion) -> lsp::CompletionItem {
    use ide::CompletionKind as K;
    let kind = match completion.kind {
        K::Keyword => lsp::CompletionItemKind::KEYWORD,
        K::Snippet => lsp::CompletionItemKind::SNIPPET,
        K::Module => lsp::CompletionItemKind::MODULE,
        K::Function => lsp::CompletionItemKind::FUNCTION,
        K::Constant => lsp::CompletionItemKind::CONSTANT,
        K::Enum => lsp::CompletionItemKind::ENUM,
        K::Variant => lsp::CompletionItemKind::ENUM_MEMBER,
        K::State => lsp::CompletionItemKind::STRUCT,
        K::Field => lsp::CompletionItemKind::FIELD,
        K::Flow => lsp::CompletionItemKind::EVENT,
        K::Attribute => lsp::CompletionItemKind::PROPERTY,
        K::Variable => lsp::CompletionItemKind::VARIABLE,
    };
    lsp::CompletionItem {
        label: completion.label,
        kind: Some(kind),
        detail: completion.detail,
        documentation: completion.documentation.map(|value| {
            lsp::Documentation::MarkupContent(lsp::MarkupContent {
                kind: lsp::MarkupKind::Markdown,
                value,
            })
        }),
        insert_text: completion.insert,
        insert_text_format: Some(if completion.snippet {
            lsp::InsertTextFormat::SNIPPET
        } else {
            lsp::InsertTextFormat::PLAIN_TEXT
        }),
        ..Default::default()
    }
}

#[allow(deprecated)]
fn document_symbol(analysis: &Analysis, file: FileId, symbol: ide::Symbol) -> lsp::DocumentSymbol {
    use sand_lang::DefKind as D;
    let kind = match symbol.kind {
        D::Module => lsp::SymbolKind::MODULE,
        D::Const => lsp::SymbolKind::CONSTANT,
        D::Enum => lsp::SymbolKind::ENUM,
        D::Variant => lsp::SymbolKind::ENUM_MEMBER,
        D::State => lsp::SymbolKind::STRUCT,
        D::Field => lsp::SymbolKind::FIELD,
        D::Function | D::ExternFunction => lsp::SymbolKind::FUNCTION,
        D::Flow | D::Event => lsp::SymbolKind::EVENT,
        D::Local => lsp::SymbolKind::VARIABLE,
        D::CustomItem | D::Recipe | D::Tag | D::BossBar | D::Loot => lsp::SymbolKind::CONSTANT,
    };
    lsp::DocumentSymbol {
        name: symbol.name,
        detail: symbol.detail,
        kind,
        tags: None,
        deprecated: None,
        range: range(analysis, file, symbol.span),
        selection_range: range(analysis, file, symbol.selection),
        children: Some(
            symbol
                .children
                .into_iter()
                .map(|child| document_symbol(analysis, file, child))
                .collect(),
        ),
    }
}

/// Convert a `file:` URI to a filesystem path.
pub fn uri_to_path(uri: &lsp::Uri) -> Option<PathBuf> {
    let text = uri.as_str();
    let rest = text.strip_prefix("file://")?;
    // Windows URIs look like file:///C:/dir; drop the slash before the drive.
    let rest = match rest.as_bytes() {
        [b'/', drive, b':', ..] if drive.is_ascii_alphabetic() && cfg!(windows) => &rest[1..],
        _ => rest,
    };
    let mut bytes = Vec::with_capacity(rest.len());
    let raw = rest.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%'
            && i + 2 < raw.len()
            && let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&raw[i + 1..i + 3]).ok()?, 16)
        {
            bytes.push(byte);
            i += 3;
            continue;
        }
        bytes.push(raw[i]);
        i += 1;
    }
    Some(PathBuf::from(String::from_utf8(bytes).ok()?))
}

/// Convert a filesystem path to a `file:` URI.
pub fn path_to_uri(path: &Path) -> Option<lsp::Uri> {
    let text = path.to_str()?.replace('\\', "/");
    let mut out = String::from("file://");
    if !text.starts_with('/') {
        out.push('/');
    }
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-_.~:".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    lsp::Uri::from_str(&out).ok()
}
