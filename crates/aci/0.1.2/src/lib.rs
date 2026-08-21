use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use bytes::Bytes;
use http::{HeaderMap, Method, Request, Response, StatusCode};
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use openapiv3::{
    OpenAPI, Operation, Parameter, ParameterSchemaOrContent, PathItem, ReferenceOr, Schema,
    SchemaKind, Type,
};
use reqwest::Url;
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value};
use thiserror::Error;
use tower::Service;
use tower::ServiceExt;

#[derive(Debug, Error)]
pub enum AciError {
    #[error("invalid argument: {0}")]
    InvalidArg(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("openapi error: {0}")]
    OpenApi(String),
    #[error("service error: {0}")]
    Service(String),
}

#[derive(Debug, Clone)]
pub struct FetchInput {
    pub body: Option<String>,
    pub headers: HeaderMap,
    pub method: Method,
    pub path: String,
    pub query: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub body: Vec<u8>,
    pub headers: HeaderMap,
    pub method: Method,
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub body: Vec<u8>,
    pub headers: HeaderMap,
    pub status: StatusCode,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stderr: String,
    pub stdout: String,
}

impl CommandOutput {
    fn ok(stdout: String) -> Self {
        Self {
            exit_code: 0,
            stderr: String::new(),
            stdout,
        }
    }

    fn err(stderr: String, code: i32) -> Self {
        Self {
            exit_code: code,
            stderr,
            stdout: String::new(),
        }
    }
}

#[async_trait(?Send)]
trait FetchExecutor: Send {
    async fn execute(&self, request: FetchRequest) -> Result<FetchResponse, AciError>;
}

#[derive(Clone)]
pub struct MountTarget {
    inner: Arc<dyn FetchExecutor>,
}

impl MountTarget {
    pub fn remote(base_url: impl AsRef<str>) -> Result<Self, AciError> {
        Self::remote_with_timeout(base_url, None)
    }

    pub fn remote_with_timeout(
        base_url: impl AsRef<str>,
        timeout_ms: Option<u64>,
    ) -> Result<Self, AciError> {
        let base_url = Url::parse(base_url.as_ref())?;
        let mut builder = reqwest::Client::builder();
        if let Some(ms) = timeout_ms {
            builder = builder.timeout(Duration::from_millis(ms));
        }
        let client = builder.build()?;
        Ok(Self {
            inner: Arc::new(RemoteExecutor { base_url, client }),
        })
    }

    pub fn tower<S, B>(service: S) -> Self
    where
        S: Service<Request<Body>, Response = Response<B>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        S::Error: std::error::Error + Send + Sync + 'static,
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(TowerExecutor {
                service,
                _marker: PhantomData,
            }),
        }
    }
}

struct RemoteExecutor {
    base_url: Url,
    client: reqwest::Client,
}

#[async_trait(?Send)]
impl FetchExecutor for RemoteExecutor {
    async fn execute(&self, request: FetchRequest) -> Result<FetchResponse, AciError> {
        let final_url = join_remote_url(&self.base_url, &request.url)?;
        let mut req = self
            .client
            .request(request.method.clone(), final_url)
            .headers(request.headers.clone());
        if !request.body.is_empty() {
            req = req.body(request.body);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.bytes().await?.to_vec();
        Ok(FetchResponse {
            body,
            headers,
            status,
        })
    }
}

fn join_remote_url(base: &Url, req_url: &Url) -> Result<Url, AciError> {
    let mut url = base.clone();
    url.set_path(req_url.path());
    url.set_query(req_url.query());
    Ok(url)
}

struct TowerExecutor<S, B> {
    service: S,
    _marker: PhantomData<B>,
}

#[async_trait(?Send)]
impl<S, B> FetchExecutor for TowerExecutor<S, B>
where
    S: Service<Request<Body>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    async fn execute(&self, request: FetchRequest) -> Result<FetchResponse, AciError> {
        let mut req_builder = Request::builder().method(request.method).uri(
            request.url.path().to_string()
                + &request
                    .url
                    .query()
                    .map(|q| format!("?{q}"))
                    .unwrap_or_default(),
        );

        if let Some(headers) = req_builder.headers_mut() {
            *headers = request.headers;
        }

        let req = req_builder
            .body(Body::from(request.body))
            .map_err(|e| AciError::Service(e.to_string()))?;

        let response = self
            .service
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| AciError::Service(e.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .into_body()
            .collect()
            .await
            .map_err(|e| AciError::Service(e.to_string()))?
            .to_bytes()
            .to_vec();

        Ok(FetchResponse {
            body,
            headers,
            status,
        })
    }
}

#[derive(Debug, Clone)]
pub enum OpenApiSpecSource {
    Path(PathBuf),
    Text(String),
}

#[derive(Debug, Clone)]
enum ScalarKind {
    Bool,
    F64,
    I64,
    String,
}

#[derive(Debug, Clone)]
struct ParamDef {
    kind: ScalarKind,
    name: String,
    required: bool,
}

#[derive(Debug, Clone)]
struct OperationDef {
    body_params: Vec<ParamDef>,
    method: Method,
    name: String,
    path_params: Vec<ParamDef>,
    path_template: String,
    query_params: Vec<ParamDef>,
}

#[derive(Clone)]
pub struct Mount {
    base_path: Option<String>,
    name: String,
    openapi_ops: HashMap<String, OperationDef>,
    target: MountTarget,
}

impl Mount {
    pub fn fetch(name: impl Into<String>, target: MountTarget) -> Self {
        Self {
            base_path: None,
            name: name.into(),
            openapi_ops: HashMap::new(),
            target,
        }
    }

    pub fn fetch_openapi(
        name: impl Into<String>,
        target: MountTarget,
        source: OpenApiSpecSource,
    ) -> Result<Self, AciError> {
        let mut mount = Self::fetch(name, target);
        mount.openapi_ops = load_openapi_commands(&source)?;
        Ok(mount)
    }

    pub fn base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    async fn run(&self, args: &[String], verbose: bool) -> Result<CommandOutput, AciError> {
        if !self.openapi_ops.is_empty() && !args.is_empty() {
            let op_name = &args[0];
            if op_name != "raw" {
                if let Some(op) = self.openapi_ops.get(op_name) {
                    return self.run_openapi(op, &args[1..], verbose).await;
                }
                return Ok(CommandOutput::err(
                    json_error(
                        "COMMAND_NOT_FOUND",
                        format!(
                            "unknown operation '{op_name}' for mount '{}'; use 'raw' or a generated operation",
                            self.name
                        ),
                    ),
                    2,
                ));
            }
            return self.run_raw(&args[1..], verbose).await;
        }
        self.run_raw(args, verbose).await
    }

    async fn run_raw(&self, args: &[String], verbose: bool) -> Result<CommandOutput, AciError> {
        let input = parse_fetch_argv(args)?;
        let request = build_request(input, self.base_path.as_deref())?;
        let response = self.target.inner.execute(request).await?;
        Ok(format_response(response, verbose))
    }

    async fn run_openapi(
        &self,
        operation: &OperationDef,
        args: &[String],
        verbose: bool,
    ) -> Result<CommandOutput, AciError> {
        let parsed = parse_openapi_args(args)?;

        if parsed.positional.len() < operation.path_params.len() {
            return Ok(CommandOutput::err(
                json_error(
                    "VALIDATION_ERROR",
                    format!(
                        "operation '{}' requires {} positional arguments",
                        operation.name,
                        operation.path_params.len()
                    ),
                ),
                2,
            ));
        }

        let mut path = operation.path_template.clone();
        for (idx, param) in operation.path_params.iter().enumerate() {
            let value = parsed.positional[idx].clone();
            let parsed_value = parse_scalar(&param.kind, &value).map_err(|e| {
                AciError::InvalidArg(format!("invalid path param '{}': {e}", param.name))
            })?;
            path = path.replace(
                &format!("{{{}}}", param.name),
                &scalar_to_path(&parsed_value),
            );
        }

        let mut query = Vec::new();
        for param in &operation.query_params {
            match parsed.options.get(&param.name) {
                Some(v) => {
                    let value = parse_scalar(&param.kind, v).map_err(|e| {
                        AciError::InvalidArg(format!(
                            "invalid query option '--{}': {e}",
                            param.name
                        ))
                    })?;
                    query.push((param.name.clone(), scalar_to_path(&value)));
                }
                None if param.required => {
                    return Ok(CommandOutput::err(
                        json_error(
                            "VALIDATION_ERROR",
                            format!("missing required option '--{}'", param.name),
                        ),
                        2,
                    ));
                }
                None => {}
            }
        }

        let mut body_obj = JsonMap::new();
        for param in &operation.body_params {
            match parsed.options.get(&param.name) {
                Some(v) => {
                    let value = parse_scalar(&param.kind, v).map_err(|e| {
                        AciError::InvalidArg(format!("invalid body option '--{}': {e}", param.name))
                    })?;
                    body_obj.insert(param.name.clone(), value);
                }
                None if param.required => {
                    return Ok(CommandOutput::err(
                        json_error(
                            "VALIDATION_ERROR",
                            format!("missing required option '--{}'", param.name),
                        ),
                        2,
                    ));
                }
                None => {}
            }
        }

        let body = if body_obj.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&Value::Object(body_obj))?)
        };

        let mut headers = HeaderMap::new();
        if body.is_some() {
            headers.insert(
                http::header::CONTENT_TYPE,
                "application/json".parse().expect("valid header value"),
            );
        }

        let request = build_request(
            FetchInput {
                body,
                headers,
                method: operation.method.clone(),
                path,
                query,
            },
            self.base_path.as_deref(),
        )?;

        let response = self.target.inner.execute(request).await?;
        Ok(format_response(response, verbose))
    }
}

#[derive(Clone)]
pub struct AciApp {
    mounts: HashMap<String, Mount>,
    name: String,
}

impl AciApp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            mounts: HashMap::new(),
            name: name.into(),
        }
    }

    pub fn mount(mut self, mount: Mount) -> Self {
        self.mounts.insert(mount.name.clone(), mount);
        self
    }

    pub async fn run<T>(&self, argv: T, verbose: bool) -> Result<CommandOutput, AciError>
    where
        T: IntoIterator,
        T::Item: Into<String>,
    {
        let args: Vec<String> = argv.into_iter().map(Into::into).collect();
        if args.is_empty() {
            return Ok(CommandOutput::err(
                json_error(
                    "USAGE",
                    format!(
                        "usage: {} <mount> [raw path/flags | operation [args/options]]",
                        self.name
                    ),
                ),
                2,
            ));
        }

        let mount_name = &args[0];
        let mount = self
            .mounts
            .get(mount_name)
            .ok_or_else(|| AciError::Config(format!("mount '{mount_name}' is not configured")))?;
        mount.run(&args[1..], verbose).await
    }
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    mounts: Vec<MountConfig>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MountConfig {
    base_path: Option<String>,
    base_url: Option<String>,
    kind: String,
    name: String,
    openapi: Option<String>,
    timeout_ms: Option<u64>,
}

pub async fn run_cli<T>(argv: T) -> Result<CommandOutput, AciError>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    let args: Vec<String> = argv.into_iter().map(Into::into).collect();
    if args.is_empty() {
        return Ok(CommandOutput::err(
            json_error("USAGE", cli_usage_text().to_string()),
            2,
        ));
    }

    if args.len() == 1 && matches!(args[0].as_str(), "help" | "--help" | "-h") {
        return Ok(CommandOutput::ok(cli_help_text().to_string()));
    }

    if args[0] == "skills" {
        if args.len() > 1 {
            return Ok(CommandOutput::err(
                json_error("USAGE", "usage: aci skills".to_string()),
                2,
            ));
        }
        return Ok(CommandOutput::ok(skills_guide_text().to_string()));
    }

    if args[0] == "--skills" {
        return Ok(CommandOutput::err(
            json_error(
                "USAGE",
                "'--skills' is not supported. use 'aci skills'".to_string(),
            ),
            2,
        ));
    }

    if args[0] == "call" {
        return run_call_mode(&args[1..]).await;
    }

    let mut idx = 0;
    let mut config_path = "aci.toml".to_string();
    let mut verbose = false;
    while idx < args.len() {
        let token = &args[idx];
        if token == "--config" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| AciError::InvalidArg("--config requires a value".to_string()))?;
            config_path = value.clone();
            idx += 2;
            continue;
        }
        if token == "--verbose" {
            verbose = true;
            idx += 1;
            continue;
        }
        break;
    }

    let run_args = args[idx..].to_vec();
    if run_args.is_empty() {
        return Ok(CommandOutput::err(
            json_error("USAGE", "missing mount name".to_string()),
            2,
        ));
    }

    let app = load_app_from_toml(Path::new(&config_path))?;
    app.run(run_args, verbose).await
}

fn skills_guide_text() -> &'static str {
    r#"aci skills (for coding agents)

Use aci in three modes:
1) Generic API call:
   aci call --url <base_url> <path...> [flags]

2) Config-driven mount:
   aci --config aci.toml <mount> [operation|raw ...]

3) OpenAPI operation calls:
   aci --config aci.toml <mount> <operationId> [--option value]

Auth pattern:
  -H "Authorization: Bearer $TOKEN"
  -H "Accept: application/json"
  -H "User-Agent: aci"

Common checks:
- If API returns 404, verify --base-path and path segments.
- For GitHub API, always set User-Agent.
- Use '<mount> raw ...' when OpenAPI operation mapping is not enough.
"#
}

async fn run_call_mode(args: &[String]) -> Result<CommandOutput, AciError> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(CommandOutput::ok(call_help_text().to_string()));
    }

    let mut idx = 0;
    let mut base_url: Option<String> = None;
    let mut base_path: Option<String> = None;
    let mut verbose = false;
    let mut passthrough = Vec::<String>::new();

    while idx < args.len() {
        let token = &args[idx];
        match token.as_str() {
            "--url" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| AciError::InvalidArg("--url requires a value".to_string()))?;
                base_url = Some(value.clone());
                idx += 2;
            }
            "--base-path" => {
                let value = args.get(idx + 1).ok_or_else(|| {
                    AciError::InvalidArg("--base-path requires a value".to_string())
                })?;
                base_path = Some(value.clone());
                idx += 2;
            }
            "--verbose" => {
                verbose = true;
                idx += 1;
            }
            _ => {
                passthrough.push(token.clone());
                idx += 1;
            }
        }
    }

    let base_url = base_url
        .ok_or_else(|| AciError::InvalidArg("call mode requires --url <base_url>".to_string()))?;

    let target = MountTarget::remote(base_url)?;
    let mut mount = Mount::fetch("call", target);
    if let Some(bp) = base_path {
        mount = mount.base_path(bp);
    }

    mount.run_raw(&passthrough, verbose).await
}

fn cli_usage_text() -> &'static str {
    "usage: aci skills | aci call --url <base_url> [args...] | aci [--config aci.toml] <mount> ..."
}

fn cli_help_text() -> &'static str {
    r#"aci help

Usage:
  aci skills
  aci call --url <base_url> <path...> [flags]
  aci [--config aci.toml] <mount> [operation|raw ...]
  aci --help
"#
}

fn call_help_text() -> &'static str {
    r#"aci call help

Usage:
  aci call --url <base_url> <path...> [flags]

Flags:
  --url <base_url>        Base URL for the API (required)
  --base-path <prefix>    Prefix path added before request path
  --verbose               Include response headers
  -X, --method <METHOD>   HTTP method (GET by default)
  -H, --header <K: V>     Request header (repeatable)
  -d, --data <json>       JSON request body
  --body <json>           Alias of --data
  --query <k=v>           Query parameter (repeatable)
"#
}

fn load_app_from_toml(path: &Path) -> Result<AciApp, AciError> {
    let raw = std::fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&raw)?;
    let mut app = AciApp::new(config.name.unwrap_or_else(|| "aci".to_string()));

    for mount in config.mounts {
        if mount.kind != "remote" {
            return Err(AciError::Config(format!(
                "mount '{}' has unsupported kind '{}'; only 'remote' is supported in aci.toml",
                mount.name, mount.kind
            )));
        }

        let base_url = mount
            .base_url
            .ok_or_else(|| AciError::Config(format!("mount '{}' requires base_url", mount.name)))?;

        let target = MountTarget::remote_with_timeout(base_url, mount.timeout_ms)?;
        let mut m = if let Some(openapi_path) = mount.openapi {
            Mount::fetch_openapi(
                mount.name,
                target,
                OpenApiSpecSource::Path(PathBuf::from(openapi_path)),
            )?
        } else {
            Mount::fetch(mount.name, target)
        };

        if let Some(base_path) = mount.base_path {
            m = m.base_path(base_path);
        }

        app = app.mount(m);
    }

    Ok(app)
}

pub fn parse_fetch_argv(args: &[String]) -> Result<FetchInput, AciError> {
    let mut segments = Vec::<String>::new();
    let mut headers = HeaderMap::new();
    let mut query = Vec::<(String, String)>::new();
    let mut method: Option<Method> = None;
    let mut body: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let token = &args[i];

        if token.starts_with("--") {
            let body_key = token.strip_prefix("--").unwrap_or(token);
            if let Some((key, value)) = body_key.split_once('=') {
                handle_long_flag(
                    key,
                    value.to_string(),
                    &mut method,
                    &mut body,
                    &mut headers,
                    &mut query,
                )?;
                i += 1;
                continue;
            }

            let key = body_key.to_string();
            let value = args
                .get(i + 1)
                .ok_or_else(|| AciError::InvalidArg(format!("flag '--{key}' requires a value")))?
                .clone();

            handle_long_flag(
                &key,
                value,
                &mut method,
                &mut body,
                &mut headers,
                &mut query,
            )?;
            i += 2;
            continue;
        }

        if token.starts_with('-') && token.len() == 2 {
            let value = args.get(i + 1).ok_or_else(|| {
                AciError::InvalidArg(format!("flag '{}' requires a value", token))
            })?;
            match token.as_str() {
                "-X" => {
                    method = Some(parse_method(value)?);
                    i += 2;
                }
                "-d" => {
                    body = Some(value.clone());
                    i += 2;
                }
                "-H" => {
                    let (k, v) = parse_header(value)?;
                    headers.insert(
                        http::header::HeaderName::from_bytes(k.as_bytes())
                            .map_err(|e| AciError::InvalidArg(e.to_string()))?,
                        http::HeaderValue::from_str(&v)
                            .map_err(|e| AciError::InvalidArg(e.to_string()))?,
                    );
                    i += 2;
                }
                _ => {
                    return Err(AciError::InvalidArg(format!(
                        "unknown short flag '{}'",
                        token
                    )));
                }
            }
            continue;
        }

        segments.push(token.clone());
        i += 1;
    }

    let final_method = method.unwrap_or_else(|| {
        if body.is_some() {
            Method::POST
        } else {
            Method::GET
        }
    });

    Ok(FetchInput {
        body,
        headers,
        method: final_method,
        path: if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        },
        query,
    })
}

fn handle_long_flag(
    key: &str,
    value: String,
    method: &mut Option<Method>,
    body: &mut Option<String>,
    headers: &mut HeaderMap,
    query: &mut Vec<(String, String)>,
) -> Result<(), AciError> {
    match key {
        "method" => {
            *method = Some(parse_method(&value)?);
        }
        "body" | "data" => {
            *body = Some(value);
        }
        "header" => {
            let (k, v) = parse_header(&value)?;
            headers.insert(
                http::header::HeaderName::from_bytes(k.as_bytes())
                    .map_err(|e| AciError::InvalidArg(e.to_string()))?,
                http::HeaderValue::from_str(&v).map_err(|e| AciError::InvalidArg(e.to_string()))?,
            );
        }
        "query" => {
            let (k, v) = value
                .split_once('=')
                .ok_or_else(|| AciError::InvalidArg("--query requires KEY=VALUE".to_string()))?;
            query.push((k.to_string(), v.to_string()));
        }
        _ => {
            query.push((key.to_string(), value));
        }
    }
    Ok(())
}

fn parse_method(value: &str) -> Result<Method, AciError> {
    Method::from_bytes(value.to_uppercase().as_bytes())
        .map_err(|e| AciError::InvalidArg(format!("invalid method '{}': {e}", value)))
}

fn parse_header(value: &str) -> Result<(String, String), AciError> {
    let (k, v) = value
        .split_once(':')
        .ok_or_else(|| AciError::InvalidArg("header must be 'Key: Value'".to_string()))?;
    Ok((k.trim().to_string(), v.trim().to_string()))
}

fn build_request(input: FetchInput, base_path: Option<&str>) -> Result<FetchRequest, AciError> {
    let mut path = input.path;
    if let Some(prefix) = base_path {
        path = format!("{}{}", prefix.trim_end_matches('/'), path);
    }

    let mut url = Url::parse("http://aci.local")?;
    url.set_path(&path);
    for (k, v) in &input.query {
        url.query_pairs_mut().append_pair(k, v);
    }

    let mut headers = input.headers;
    let body_bytes = match input.body {
        Some(body) => {
            if !headers.contains_key(http::header::CONTENT_TYPE) {
                headers.insert(
                    http::header::CONTENT_TYPE,
                    "application/json".parse().expect("valid header value"),
                );
            }
            body.into_bytes()
        }
        None => Vec::new(),
    };

    Ok(FetchRequest {
        body: body_bytes,
        headers,
        method: input.method,
        url,
    })
}

fn format_response(response: FetchResponse, verbose: bool) -> CommandOutput {
    let status = response.status;
    let content_type = response
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let is_stream = content_type.contains("application/x-ndjson")
        || content_type.contains("application/ndjson");

    if is_stream {
        let text = String::from_utf8_lossy(&response.body).to_string();
        if status.is_success() {
            return CommandOutput::ok(text);
        }
        return CommandOutput::err(
            json_error(
                format!("HTTP_{}", status.as_u16()),
                if text.trim().is_empty() {
                    format!("HTTP {}", status.as_u16())
                } else {
                    text
                },
            ),
            1,
        );
    }

    let parsed = parse_body_value(&response.body);

    if status.is_success() {
        if verbose {
            let mut headers_json = JsonMap::new();
            for (k, v) in &response.headers {
                headers_json.insert(
                    k.to_string(),
                    Value::String(v.to_str().unwrap_or_default().to_string()),
                );
            }
            let mut env = JsonMap::new();
            env.insert("ok".to_string(), Value::Bool(true));
            env.insert("status".to_string(), Value::Number(status.as_u16().into()));
            env.insert("headers".to_string(), Value::Object(headers_json));
            env.insert("data".to_string(), parsed.clone());
            return CommandOutput::ok(pretty_json(&Value::Object(env)));
        }
        return CommandOutput::ok(render_value(&parsed));
    }

    let message = match &parsed {
        Value::Object(map) => map
            .get("message")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("HTTP {}", status.as_u16())),
        Value::String(s) if !s.trim().is_empty() => s.clone(),
        _ => format!("HTTP {}", status.as_u16()),
    };

    CommandOutput::err(json_error(format!("HTTP_{}", status.as_u16()), message), 1)
}

fn parse_body_value(body: &[u8]) -> Value {
    if body.is_empty() {
        return Value::Null;
    }
    match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(_) => Value::String(String::from_utf8_lossy(body).to_string()),
    }
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()) + "\n"
}

fn render_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("{s}\n"),
        _ => pretty_json(value),
    }
}

fn json_error(code: impl Into<String>, message: impl Into<String>) -> String {
    let mut obj = JsonMap::new();
    obj.insert("code".to_string(), Value::String(code.into()));
    obj.insert("message".to_string(), Value::String(message.into()));
    pretty_json(&Value::Object(obj))
}

#[derive(Debug)]
struct ParsedOpenApiArgs {
    options: HashMap<String, String>,
    positional: Vec<String>,
}

fn parse_openapi_args(args: &[String]) -> Result<ParsedOpenApiArgs, AciError> {
    let mut positional = Vec::new();
    let mut options = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let token = &args[i];
        if let Some(key) = token.strip_prefix("--") {
            if let Some((k, v)) = key.split_once('=') {
                options.insert(normalize_option_name(k), v.to_string());
                i += 1;
                continue;
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| AciError::InvalidArg(format!("flag '--{key}' requires a value")))?
                .clone();
            options.insert(normalize_option_name(key), value);
            i += 2;
            continue;
        }
        positional.push(token.clone());
        i += 1;
    }

    Ok(ParsedOpenApiArgs {
        options,
        positional,
    })
}

fn normalize_option_name(name: &str) -> String {
    name.replace('-', "_")
}

fn parse_scalar(kind: &ScalarKind, value: &str) -> Result<Value, String> {
    match kind {
        ScalarKind::String => Ok(Value::String(value.to_string())),
        ScalarKind::Bool => value
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|_| "expected boolean".to_string()),
        ScalarKind::I64 => value
            .parse::<i64>()
            .map(|v| Value::Number(v.into()))
            .map_err(|_| "expected integer".to_string()),
        ScalarKind::F64 => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| "expected number".to_string()),
    }
}

fn scalar_to_path(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

fn load_openapi_commands(
    source: &OpenApiSpecSource,
) -> Result<HashMap<String, OperationDef>, AciError> {
    let text = match source {
        OpenApiSpecSource::Path(path) => std::fs::read_to_string(path)?,
        OpenApiSpecSource::Text(text) => text.clone(),
    };

    let doc: OpenAPI = if text.trim_start().starts_with('{') {
        serde_json::from_str(&text)?
    } else {
        serde_yaml::from_str(&text)?
    };

    let mut commands = HashMap::new();

    for (path, item) in doc.paths.paths {
        let item = deref_path_item(item)?;
        for (method, operation) in collect_operations(item) {
            let name = operation_name(&method, &path, &operation);
            let (path_params, query_params) = collect_parameters(&operation)?;
            let body_params = collect_body_parameters(&operation)?;
            let def = OperationDef {
                body_params,
                method,
                name: name.clone(),
                path_params,
                path_template: path.clone(),
                query_params,
            };
            if commands.insert(name.clone(), def).is_some() {
                return Err(AciError::OpenApi(format!(
                    "duplicate operation command name '{}'",
                    name
                )));
            }
        }
    }

    Ok(commands)
}

fn deref_path_item(item: ReferenceOr<PathItem>) -> Result<PathItem, AciError> {
    match item {
        ReferenceOr::Item(i) => Ok(i),
        ReferenceOr::Reference { reference } => Err(AciError::OpenApi(format!(
            "external or unresolved path reference '{}' is not supported in v1",
            reference
        ))),
    }
}

fn collect_operations(item: PathItem) -> Vec<(Method, Operation)> {
    let mut ops = Vec::new();
    if let Some(op) = item.get {
        ops.push((Method::GET, op));
    }
    if let Some(op) = item.post {
        ops.push((Method::POST, op));
    }
    if let Some(op) = item.put {
        ops.push((Method::PUT, op));
    }
    if let Some(op) = item.patch {
        ops.push((Method::PATCH, op));
    }
    if let Some(op) = item.delete {
        ops.push((Method::DELETE, op));
    }
    if let Some(op) = item.head {
        ops.push((Method::HEAD, op));
    }
    if let Some(op) = item.options {
        ops.push((Method::OPTIONS, op));
    }
    ops
}

fn operation_name(method: &Method, path: &str, operation: &Operation) -> String {
    if let Some(id) = &operation.operation_id {
        return id.clone();
    }

    let cleaned = path
        .replace('/', "_")
        .replace(['{', '}'], "")
        .trim_matches('_')
        .to_string();
    format!("{}_{}", method.as_str().to_lowercase(), cleaned)
}

fn collect_parameters(operation: &Operation) -> Result<(Vec<ParamDef>, Vec<ParamDef>), AciError> {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();

    for param in &operation.parameters {
        let param = match param {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { .. } => continue,
        };

        match param {
            Parameter::Path {
                parameter_data,
                style: _,
            } => {
                let kind = parameter_kind(&parameter_data.format)?;
                path_params.push(ParamDef {
                    kind,
                    name: normalize_option_name(&parameter_data.name),
                    required: true,
                });
            }
            Parameter::Query {
                parameter_data,
                allow_reserved: _,
                style: _,
                allow_empty_value: _,
            } => {
                let kind = parameter_kind(&parameter_data.format)?;
                query_params.push(ParamDef {
                    kind,
                    name: normalize_option_name(&parameter_data.name),
                    required: parameter_data.required,
                });
            }
            _ => {}
        }
    }

    Ok((path_params, query_params))
}

fn collect_body_parameters(operation: &Operation) -> Result<Vec<ParamDef>, AciError> {
    let Some(body) = &operation.request_body else {
        return Ok(Vec::new());
    };

    let body = match body {
        ReferenceOr::Item(item) => item,
        ReferenceOr::Reference { .. } => return Ok(Vec::new()),
    };

    let Some(media) = body.content.get("application/json") else {
        return Ok(Vec::new());
    };

    let schema = match &media.schema {
        Some(ReferenceOr::Item(schema)) => schema,
        Some(ReferenceOr::Reference { .. }) => return Ok(Vec::new()),
        None => return Ok(Vec::new()),
    };

    let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind else {
        return Ok(vec![ParamDef {
            kind: ScalarKind::String,
            name: "body".to_string(),
            required: body.required,
        }]);
    };

    let required = &obj.required;
    let mut out = Vec::new();

    for (name, value) in &obj.properties {
        let kind = match value {
            ReferenceOr::Item(schema) => schema_to_kind(schema),
            ReferenceOr::Reference { .. } => ScalarKind::String,
        };
        out.push(ParamDef {
            kind,
            name: normalize_option_name(name),
            required: required.contains(name),
        });
    }

    Ok(out)
}

fn parameter_kind(format: &ParameterSchemaOrContent) -> Result<ScalarKind, AciError> {
    match format {
        ParameterSchemaOrContent::Schema(schema) => match schema {
            ReferenceOr::Item(schema) => Ok(schema_to_kind(schema)),
            ReferenceOr::Reference { .. } => Ok(ScalarKind::String),
        },
        ParameterSchemaOrContent::Content(_) => Ok(ScalarKind::String),
    }
}

fn schema_to_kind(schema: &Schema) -> ScalarKind {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Boolean(_)) => ScalarKind::Bool,
        SchemaKind::Type(Type::Integer(_)) => ScalarKind::I64,
        SchemaKind::Type(Type::Number(_)) => ScalarKind::F64,
        _ => ScalarKind::String,
    }
}

pub async fn run_embedded<T>(
    app: &AciApp,
    args: T,
    verbose: bool,
) -> Result<CommandOutput, AciError>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    app.run(args, verbose).await
}

pub async fn with_axum_router<T, F, Fut>(build: F) -> Result<T, AciError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, AciError>>,
{
    build().await
}

pub type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, extract::Path as AxumPath, routing::get, routing::post};
    use serde_json::json;
    use tempfile::TempDir;
    use tokio::net::TcpListener;

    #[test]
    fn parse_fetch_defaults() {
        let parsed = parse_fetch_argv(&["users".to_string()]).expect("parse success");
        assert_eq!(parsed.path, "/users");
        assert_eq!(parsed.method, Method::GET);
    }

    #[test]
    fn parse_fetch_body_implies_post() {
        let parsed = parse_fetch_argv(&[
            "users".to_string(),
            "-d".to_string(),
            "{\"name\":\"Bob\"}".to_string(),
        ])
        .expect("parse success");
        assert_eq!(parsed.method, Method::POST);
        assert!(parsed.body.is_some());
    }

    #[tokio::test]
    async fn inprocess_mount_works() {
        let app = Router::new()
            .route(
                "/users",
                get(|| async { Json(json!({ "users": [{"id": 1, "name": "Alice"}] })) }),
            )
            .route(
                "/users/{id}",
                get(|AxumPath(id): AxumPath<i64>| async move { Json(json!({ "id": id })) }),
            )
            .route(
                "/users",
                post(|| async { Json(json!({ "created": true })) }),
            );

        let target = MountTarget::tower(app);
        let mount = Mount::fetch("api", target);
        let app = AciApp::new("aci").mount(mount);

        let out = app
            .run(vec!["api", "users"], false)
            .await
            .expect("run success");

        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.contains("Alice"));
    }

    #[tokio::test]
    async fn openapi_command_works() {
        let spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users/{id}:
    get:
      operationId: getUser
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: integer
      responses:
        '200':
          description: OK
"#;

        let app =
            Router::new().route(
                "/users/{id}",
                get(|AxumPath(id): AxumPath<i64>| async move {
                    Json(json!({ "id": id, "name": "Alice" }))
                }),
            );

        let target = MountTarget::tower(app);
        let mount = Mount::fetch_openapi("api", target, OpenApiSpecSource::Text(spec.to_string()))
            .expect("openapi load success");
        let app = AciApp::new("aci").mount(mount);

        let out = app
            .run(vec!["api", "getUser", "42"], false)
            .await
            .expect("run success");
        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.contains("42"));
    }

    async fn spawn_server(app: Router) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind listener");
        let addr = listener.local_addr().expect("read addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn call_mode_remote_works() {
        let app = Router::new().route("/health", get(|| async { Json(json!({ "ok": true })) }));
        let base_url = spawn_server(app).await;
        let out = run_cli(vec![
            "call".to_string(),
            "--url".to_string(),
            base_url,
            "health".to_string(),
        ])
        .await
        .expect("run success");
        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.contains("\"ok\": true"));
    }

    #[tokio::test]
    async fn config_mode_openapi_works() {
        let app = Router::new().route("/health", get(|| async { Json(json!({ "ok": true })) }));
        let base_url = spawn_server(app).await;
        let dir = TempDir::new().expect("tempdir");
        let spec_path = dir.path().join("openapi.yaml");
        let config_path = dir.path().join("aci.toml");

        std::fs::write(
            &spec_path,
            r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /health:
    get:
      operationId: healthCheck
      responses:
        '200':
          description: OK
"#,
        )
        .expect("write spec");

        std::fs::write(
            &config_path,
            format!(
                r#"
name = "aci"

[[mounts]]
name = "api"
kind = "remote"
base_url = "{base_url}"
openapi = "{}"
"#,
                spec_path.display()
            ),
        )
        .expect("write config");

        let out = run_cli(vec![
            "--config".to_string(),
            config_path.display().to_string(),
            "api".to_string(),
            "healthCheck".to_string(),
        ])
        .await
        .expect("run success");

        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.contains("\"ok\": true"));
    }

    #[tokio::test]
    async fn skills_subcommand_works() {
        let out = run_cli(vec!["skills".to_string()])
            .await
            .expect("run success");
        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.contains("for coding agents"));
        assert!(out.stderr.is_empty());
    }

    #[tokio::test]
    async fn skills_flag_returns_usage_error() {
        let out = run_cli(vec!["--skills".to_string()])
            .await
            .expect("run success");
        assert_eq!(out.exit_code, 2);
        assert!(out.stderr.contains("aci skills"));
        assert!(out.stdout.is_empty());
    }

    #[tokio::test]
    async fn global_help_subcommands_work() {
        for arg in ["help", "--help", "-h"] {
            let out = run_cli(vec![arg.to_string()]).await.expect("run success");
            assert_eq!(out.exit_code, 0);
            assert!(out.stdout.contains("Usage:"));
            assert!(out.stderr.is_empty());
        }
    }

    #[tokio::test]
    async fn call_help_works() {
        for arg in ["--help", "-h"] {
            let out = run_cli(vec!["call".to_string(), arg.to_string()])
                .await
                .expect("run success");
            assert_eq!(out.exit_code, 0);
            assert!(out.stdout.contains("aci call --url <base_url>"));
            assert!(out.stderr.is_empty());
        }
    }
}
