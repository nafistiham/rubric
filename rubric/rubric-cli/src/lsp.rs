//! `rubric lsp` — LSP server that publishes diagnostics over stdio.
//!
//! Editors connect to this server via the Language Server Protocol (stdio transport).
//! See docs/editor-setup.md for per-editor configuration snippets.

use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use rubric_core::{LintContext, Rule};

use crate::config::Config;
use crate::todo_config::TodoConfig;

// ── Server state ──────────────────────────────────────────────────────────────

struct ServerState {
    root_dir: PathBuf,
    #[allow(dead_code)]
    config: Config,
    todo: TodoConfig,
    rules: Vec<Box<dyn Rule + Send + Sync>>,
}

// ── Backend ───────────────────────────────────────────────────────────────────

struct Backend {
    client: Client,
    state: RwLock<Option<ServerState>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self { client, state: RwLock::new(None) }
    }

    /// Reload config + todo from disk, rebuild rules. Called on initialize and on file-watch events.
    async fn reload_config(&self, root_dir: &Path) {
        match Self::load_state(root_dir) {
            Ok(state) => {
                *self.state.write().await = Some(state);
            }
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("rubric: failed to load config: {e}"))
                    .await;
            }
        }
    }

    fn load_state(root_dir: &Path) -> Result<ServerState> {
        let config = Config::find_and_load(root_dir)?;
        let todo = TodoConfig::load(root_dir)?;
        let rules = crate::build_rules_with_config(&config);
        Ok(ServerState { root_dir: root_dir.to_path_buf(), config, todo, rules })
    }

    /// Lint `source` (the in-memory content of `uri`) and publish diagnostics.
    async fn publish_diagnostics(&self, uri: Url, source: String) {
        let state_guard = self.state.read().await;
        let Some(state) = state_guard.as_ref() else {
            return;
        };

        let path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return,
        };

        let diagnostics = lint_source(&path, &source, state);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

// ── Linting ───────────────────────────────────────────────────────────────────

/// Run all enabled cops on `source` (in-memory), apply todo suppression, and
/// return LSP-formatted diagnostics.
fn lint_source(
    path: &Path,
    source: &str,
    state: &ServerState,
) -> Vec<Diagnostic> {
    let ctx = LintContext::new(path, source);

    // Source-level checks
    let mut diags: Vec<rubric_core::Diagnostic> = state.rules
        .iter()
        .flat_map(|r| r.check_source(&ctx))
        .collect();

    // AST-level checks
    if state.rules.iter().any(|r| !r.node_kinds().is_empty()) {
        let ast_diags = rubric_core::walk(source.as_bytes(), &ctx, &state.rules);
        diags.extend(ast_diags);
    }

    // Honour # rubocop:disable directives
    let diags = rubric_core::filter_disabled_by_directives(
        source,
        diags,
        &ctx.line_start_offsets,
    );

    // Apply todo suppression
    let rel = path.strip_prefix(&state.root_dir).unwrap_or(path);
    let rel_str = rel.to_string_lossy();

    diags
        .into_iter()
        .filter(|d| !state.todo.is_suppressed(d.rule, &rel_str))
        .map(|d| to_lsp_diagnostic(&ctx, d))
        .collect()
}

/// Convert a `rubric_core::Diagnostic` to an `lsp_types::Diagnostic`.
/// `offset_to_line_col` returns 1-based; LSP uses 0-based positions.
fn to_lsp_diagnostic(ctx: &LintContext, d: rubric_core::Diagnostic) -> Diagnostic {
    let (start_line, start_col) = ctx.offset_to_line_col(d.range.start);
    let (end_line, end_col)     = ctx.offset_to_line_col(d.range.end);

    let severity = match d.severity {
        rubric_core::Severity::Error   => DiagnosticSeverity::ERROR,
        rubric_core::Severity::Warning => DiagnosticSeverity::WARNING,
        rubric_core::Severity::Info    => DiagnosticSeverity::INFORMATION,
    };

    Diagnostic {
        range: Range {
            start: Position {
                line:      (start_line.saturating_sub(1)) as u32,
                character: (start_col.saturating_sub(1))  as u32,
            },
            end: Position {
                line:      (end_line.saturating_sub(1)) as u32,
                character: (end_col.saturating_sub(1))  as u32,
            },
        },
        severity: Some(severity),
        source: Some("rubric".to_string()),
        message: format!("{} ({})", d.message, d.rule),
        ..Default::default()
    }
}

// ── LanguageServer implementation ─────────────────────────────────────────────

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        // Determine root directory
        let root_dir = params
            .root_uri
            .as_ref()
            .and_then(|u| u.to_file_path().ok())
            .or_else(|| {
                params.workspace_folders.as_ref()
                    .and_then(|wf| wf.first())
                    .and_then(|f| f.uri.to_file_path().ok())
            })
            .unwrap_or_else(|| PathBuf::from("."));

        self.reload_config(&root_dir).await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                // Diagnostics are push-only in LSP; no capability needed.
                // Watch rubric.toml and .rubric_todo.toml for config changes.
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: None,
                    file_operations: None,
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "rubric".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        // Register file watchers for rubric.toml and .rubric_todo.toml so we
        // reload config automatically when they change. Fire-and-forget via
        // tokio::spawn so we don't block initialized() waiting for the client
        // to ack the registration request (some clients / tests don't respond).
        let client = self.client.clone();
        tokio::spawn(async move {
            let registrations = vec![Registration {
                id: "rubric-config-watcher".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(
                    serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String("**/rubric.toml".to_string()),
                                kind: None,
                            },
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String(
                                    "**/.rubric_todo.toml".to_string(),
                                ),
                                kind: None,
                            },
                        ],
                    })
                    .unwrap(),
                ),
            }];
            let _ = client.register_capability(registrations).await;
        });
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.publish_diagnostics(
            params.text_document.uri,
            params.text_document.text,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Full sync — take the last content_changes entry (should be only one).
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };
        self.publish_diagnostics(params.text_document.uri, change.text).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Re-lint from disk on save so we pick up any file-system-level changes.
        let path = match params.text_document.uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return,
        };
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return,
        };
        self.publish_diagnostics(params.text_document.uri, source).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Clear diagnostics for closed files.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        // rubric.toml or .rubric_todo.toml changed — reload config.
        let root_dir = {
            let state_guard = self.state.read().await;
            state_guard.as_ref().map(|s| s.root_dir.clone())
        };
        if let Some(root) = root_dir {
            self.reload_config(&root).await;
            self.client
                .log_message(MessageType::INFO, "rubric: config reloaded")
                .await;
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the LSP server on stdin/stdout. Blocks until the client disconnects.
pub fn run() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let stdin  = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) =
            LspService::new(|client| Backend::new(client));

        Server::new(stdin, stdout, socket).serve(service).await;
    });

    Ok(())
}
