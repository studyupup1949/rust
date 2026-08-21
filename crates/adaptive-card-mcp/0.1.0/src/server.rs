//! Server setup: loads core library, registers tools/prompts/resources,
//! and runs them over the requested transport.
//!
//! The [`Server`] struct implements `rmcp::ServerHandler` directly — no
//! macros — so the wiring is explicit and transparent.

use std::borrow::Cow;
use std::future::Future;
use std::sync::LazyLock;

use adaptive_card_core::KnowledgeBase;
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, GetPromptRequestParams, GetPromptResult,
    Implementation, InitializeResult, ListPromptsResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, Prompt, PromptArgument, PromptMessage, PromptMessageRole, RawResource,
    ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
    Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value};

use crate::prompts::{self, PromptDef};
use crate::resources::{self, LISTED};
use crate::tools::{self, TOOL_NAMES, ToolCtx, schemas};

/// In-process server state shared by all transport wrappers.
#[derive(Clone)]
pub struct Server {
    pub kb: &'static KnowledgeBase,
}

impl Server {
    #[must_use]
    pub fn new(kb: &'static KnowledgeBase) -> Self {
        Self { kb }
    }

    fn ctx(&self) -> ToolCtx {
        ToolCtx { kb: self.kb }
    }
}

// ---------- static tool/prompt/resource tables -------------------------------

static TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(schemas::build_all);

static PROMPTS: LazyLock<Vec<Prompt>> = LazyLock::new(|| {
    prompts::all()
        .iter()
        .map(|p: &PromptDef| {
            let arguments: Vec<PromptArgument> = p
                .arguments
                .iter()
                .map(|a| {
                    PromptArgument::new(a.name)
                        .with_description(a.description)
                        .with_required(a.required)
                })
                .collect();
            Prompt::new(
                p.name,
                Some(p.description),
                if arguments.is_empty() {
                    None
                } else {
                    Some(arguments)
                },
            )
        })
        .collect()
});

static RESOURCES: LazyLock<Vec<Resource>> = LazyLock::new(|| {
    LISTED
        .iter()
        .map(|r| {
            let mut raw = RawResource::new(r.uri, r.name);
            raw.description = Some(r.description.to_string());
            raw.mime_type = Some(r.mime_type.to_string());
            Resource {
                raw,
                annotations: None,
            }
        })
        .collect()
});

// ---------- ServerHandler implementation -------------------------------------

#[allow(
    clippy::manual_async_fn,
    reason = "rmcp::ServerHandler method signatures require explicit `impl Future + MaybeSendFuture + '_` bounds that async-fn desugaring would not satisfy"
)]
impl ServerHandler for Server {
    fn get_info(&self) -> InitializeResult {
        let instructions = concat!(
            "Adaptive Cards v1.6 helper. Use tools to validate, analyze, ",
            "optimize, transform, templatize and generate Adaptive Cards. ",
            "Browse the knowledge base via list_examples / get_example / ",
            "suggest_layout. Resources expose the Microsoft v1.6 JSON Schema ",
            "(ac://schema/v1.6), the host capability matrix (ac://hosts), ",
            "and curated examples (ac://examples)."
        );

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .enable_resources()
            .build();

        InitializeResult::new(capabilities)
            .with_server_info(Implementation::new(
                "adaptive-card-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(instructions)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        let tools = TOOLS.clone();
        async move { Ok(ListToolsResult::with_all_items(tools)) }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + MaybeSendFuture + '_ {
        let ctx = self.ctx();
        async move {
            let name = request.name.as_ref();
            if !TOOL_NAMES.contains(&name) {
                return Err(McpError::invalid_params(
                    Cow::Owned(format!("unknown tool: {name}")),
                    None,
                ));
            }
            let args = request
                .arguments
                .map_or(Value::Object(Map::new()), Value::Object);
            match tools::dispatch(&ctx, name, args) {
                Ok(value) => Ok(CallToolResult::structured(value)),
                Err(message) => Ok(CallToolResult::error(vec![Content::text(message)])),
            }
        }
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, McpError>> + MaybeSendFuture + '_ {
        let prompts = PROMPTS.clone();
        async move { Ok(ListPromptsResult::with_all_items(prompts)) }
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, McpError>> + MaybeSendFuture + '_ {
        async move {
            let args = request.arguments.map_or(Value::Null, Value::Object);
            match prompts::render(&request.name, &args) {
                Some(text) => {
                    let description = prompts::all()
                        .iter()
                        .find(|p| p.name == request.name)
                        .map(|p| p.description.to_string());
                    let mut result = GetPromptResult::new(vec![PromptMessage::new_text(
                        PromptMessageRole::User,
                        text,
                    )]);
                    result.description = description;
                    Ok(result)
                }
                None => Err(McpError::invalid_params(
                    Cow::Owned(format!("unknown prompt: {}", request.name)),
                    None,
                )),
            }
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + MaybeSendFuture + '_ {
        let list = RESOURCES.clone();
        async move { Ok(ListResourcesResult::with_all_items(list)) }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + MaybeSendFuture + '_ {
        let kb = self.kb;
        async move {
            match resources::read(kb, &request.uri) {
                Some(body) => {
                    let text = serde_json::to_string_pretty(&body).unwrap_or_else(|_| {
                        String::from("{\"error\":\"failed to serialize resource\"}")
                    });
                    Ok(ReadResourceResult::new(vec![
                        ResourceContents::text(text, request.uri.clone())
                            .with_mime_type("application/json"),
                    ]))
                }
                None => Err(McpError::resource_not_found(
                    Cow::Owned(format!("unknown resource uri: {}", request.uri)),
                    None,
                )),
            }
        }
    }
}

// ---------- transport entry points -------------------------------------------

/// Run the MCP server over stdio transport.
pub async fn run_stdio(srv: Server) -> Result<()> {
    tracing::info!("stdio transport active — waiting for MCP client");
    let running = srv
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
        .map_err(|e| anyhow::anyhow!("failed to start stdio server: {e}"))?;
    let reason = running
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("stdio server terminated abnormally: {e}"))?;
    tracing::info!("stdio server stopped: {reason:?}");
    Ok(())
}

/// Run the MCP server over SSE transport.
///
/// Not implemented for v0.1.0: `rmcp 1.4` does not ship a plain SSE server
/// transport through the feature set we enable. A future release will add
/// an axum-based SSE adapter.
pub async fn run_sse(_srv: Server, _bind: &str, _port: u16, _api_key: Option<&str>) -> Result<()> {
    anyhow::bail!(
        "SSE transport is not supported in v0.1.0 — use --transport stdio \
         (see CLAUDE.md for known limitations)"
    )
}
