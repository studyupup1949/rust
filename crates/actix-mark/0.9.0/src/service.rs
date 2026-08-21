use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use actix_web::{
    dev::{AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse},
    http::header,
    Error, HttpResponse,
};
use actix_files::NamedFile;
use tokio::fs;

use crate::{markdown, template};

// ---------------------------------------------------------------------------
// Internal path resolution result
// ---------------------------------------------------------------------------

enum ResolvedPath {
    Markdown(PathBuf),
    Static(PathBuf),
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Actix-web service that serves a directory, rendering `.md` files as HTML.
///
/// # Example
/// ```no_run
/// use actix_mark::MarkdownFiles;
///
/// let svc = MarkdownFiles::new("/docs", "./content")
///     .template("<html><body><markdown></body></html>");
/// ```
pub struct MarkdownFiles {
    mount_path: String,
    dir: PathBuf,
    template: String,
}

impl MarkdownFiles {
    /// Create a new `MarkdownFiles` service.
    ///
    /// - `mount_path`: the URL prefix (e.g. `"/docs"`)
    /// - `dir`: the filesystem directory to serve from
    pub fn new(mount_path: &str, dir: impl Into<PathBuf>) -> Self {
        Self {
            mount_path: mount_path.to_string(),
            dir: dir.into(),
            template: String::new(),
        }
    }

    /// Set the HTML template as a string.
    ///
    /// The template must contain a `<markdown>` (or `<markdown/>`) tag which
    /// will be replaced with the rendered HTML content.
    pub fn template(mut self, tmpl: impl Into<String>) -> Self {
        self.template = tmpl.into();
        self
    }

    /// Load the HTML template from a file.
    ///
    /// The file is read eagerly so errors surface at startup, not at request time.
    pub fn template_file(mut self, path: impl AsRef<Path>) -> std::io::Result<Self> {
        self.template = std::fs::read_to_string(path)?;
        Ok(self)
    }
}

// ---------------------------------------------------------------------------
// HttpServiceFactory
// ---------------------------------------------------------------------------

impl HttpServiceFactory for MarkdownFiles {
    fn register(self, config: &mut AppService) {
        // The pattern ends with `{tail:.*}` to capture everything under the
        // mount path, including nested directories.
        let pattern = if self.mount_path.ends_with('/') {
            format!("{}{}", self.mount_path, "{tail:.*}")
        } else {
            format!("{}/{}", self.mount_path, "{tail:.*}")
        };

        let rdef = ResourceDef::new(pattern);
        let svc = MarkdownFilesService(Arc::new(MarkdownFilesInner {
            dir: self.dir,
            template: self.template,
        }));

        config.register_service(rdef, None, svc, None);
    }
}

// ---------------------------------------------------------------------------
// Inner service data and newtype service handle
// ---------------------------------------------------------------------------

struct MarkdownFilesInner {
    dir: PathBuf,
    template: String,
}

/// Newtype so we can implement foreign traits without hitting the orphan rule.
pub struct MarkdownFilesService(Arc<MarkdownFilesInner>);

// ---------------------------------------------------------------------------
// actix-web Service + ServiceFactory boilerplate
// ---------------------------------------------------------------------------

impl actix_web::dev::ServiceFactory<ServiceRequest> for MarkdownFilesService {
    type Response = ServiceResponse;
    type Error = Error;
    type Config = ();
    type Service = Self;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        std::future::ready(Ok(MarkdownFilesService(self.0.clone())))
    }
}

impl actix_web::dev::Service<ServiceRequest> for MarkdownFilesService {
    type Response = ServiceResponse;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let inner = self.0.clone();
        Box::pin(async move { inner.handle(req).await })
    }
}

// ---------------------------------------------------------------------------
// Request handling
// ---------------------------------------------------------------------------

impl MarkdownFilesInner {
    async fn handle(&self, req: ServiceRequest) -> Result<ServiceResponse, Error> {
        let tail = req
            .match_info()
            .get("tail")
            .unwrap_or("")
            .to_string();

        match self.resolve_path(&tail) {
            Some(ResolvedPath::Markdown(path)) => self.serve_markdown(req, &path).await,
            Some(ResolvedPath::Static(path)) => self.serve_static(req, &path),
            None => {
                let (req, _) = req.into_parts();
                Ok(ServiceResponse::new(req, HttpResponse::NotFound().finish()))
            }
        }
    }

    fn resolve_path(&self, tail: &str) -> Option<ResolvedPath> {
        let rel = tail.trim_start_matches('/');
        let candidate = self.dir.join(rel);

        // Safety check first — bail if the candidate escapes the base dir.
        if !is_safe(&self.dir, &candidate) {
            return None;
        }

        // Directory → try index.md
        if candidate.is_dir() {
            let index = candidate.join("index.md");
            if index.is_file() {
                return Some(ResolvedPath::Markdown(index));
            }
            return None;
        }

        // Exact .md file
        if candidate.extension().is_some_and(|e| e == "md") && candidate.is_file() {
            return Some(ResolvedPath::Markdown(candidate));
        }

        // Extension-less path → try appending .md
        if candidate.extension().is_none() {
            let with_md = candidate.with_extension("md");
            if with_md.is_file() {
                return Some(ResolvedPath::Markdown(with_md));
            }
        }

        // Anything else that exists → static file
        if candidate.is_file() {
            return Some(ResolvedPath::Static(candidate));
        }

        None
    }

    async fn serve_markdown(
        &self,
        req: ServiceRequest,
        path: &Path,
    ) -> Result<ServiceResponse, Error> {
        let source = fs::read_to_string(path).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })?;

        let html_body = markdown::to_html(&source);
        let page = template::render(&self.template, &html_body);

        let (req, _) = req.into_parts();
        let resp = HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(page);

        Ok(ServiceResponse::new(req, resp))
    }

    fn serve_static(
        &self,
        req: ServiceRequest,
        path: &Path,
    ) -> Result<ServiceResponse, Error> {
        let file = NamedFile::open(path).map_err(actix_web::error::ErrorInternalServerError)?;
        let (http_req, _) = req.into_parts();
        let resp = file.into_response(&http_req);
        Ok(ServiceResponse::new(http_req, resp))
    }
}

// ---------------------------------------------------------------------------
// Path traversal guard
// ---------------------------------------------------------------------------

fn is_safe(base: &Path, target: &Path) -> bool {
    // Canonicalize the base; for the target we build a canonical-like path
    // without requiring it to exist yet (to handle .md extension probing).
    let Ok(canon_base) = base.canonicalize() else {
        return false;
    };

    // Normalize the target by canonicalizing its existing prefix.
    // We walk up until we find a component that exists, then re-attach the rest.
    let mut existing = target.to_path_buf();
    let mut suffix = vec![];
    loop {
        if existing.exists() {
            break;
        }
        match existing.file_name() {
            Some(name) => {
                suffix.push(name.to_os_string());
                existing.pop();
            }
            None => return false,
        }
    }
    let Ok(mut canon) = existing.canonicalize() else {
        return false;
    };
    for component in suffix.into_iter().rev() {
        canon.push(component);
    }

    canon.starts_with(&canon_base)
}

// ---------------------------------------------------------------------------
// URL helpers exposed for the example/tests
// ---------------------------------------------------------------------------

/// Returns the URL path a browser should use to reach a given `.md` file,
/// stripping the `.md` extension.
pub fn md_url(base_url: &str, rel_path: &str) -> String {
    let stripped = rel_path
        .strip_suffix(".md")
        .unwrap_or(rel_path);
    format!("{}/{}", base_url.trim_end_matches('/'), stripped.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_path_within_base() {
        let base = PathBuf::from(".");
        let target = PathBuf::from("./src/lib.rs");
        assert!(is_safe(&base, &target));
    }

    #[test]
    fn traversal_is_rejected() {
        let base = PathBuf::from("./src");
        let target = PathBuf::from("./src/../../Cargo.toml");
        assert!(!is_safe(&base, &target));
    }
}
