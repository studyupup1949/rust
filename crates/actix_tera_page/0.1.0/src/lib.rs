//! This crate provides a middleware for `actix_web` that reduces the boilerplate needed to
//! create SSR websites with `Tera`. It matches GET request paths to templates and renders them
//! using a shared "base context". An example use case would be populating a website navbar
//! with user information or login/signup buttons, depending on if there is a user logged in or not.
//!
//! To get started, create an async function that accepts `HttpResponse`, returning `Context`.
//! Register a `Tera` object as app data on the web server, then wrap the application or route
//! in the middleware constructed with `TeraPage::new`.
//!
//! ```
//! struct State {
//!     name: String,
//! }
//!
//! async fn base_context(req: HttpRequest) -> Context {
//!    let state = req.app_data::<Data<State>>().unwrap();
//!    let mut context = Context::new();
//!    
//!    // This function could, for example, make SQL queries
//!    // through a connection stored in `State`.
//!    context.insert("username", &state.name);
//!
//!    context
//! }
//!
//! #[get("/complex-page")]
//! async fn complex_page(tera: web::Data<Tera>, req: HttpRequest) -> impl Responder {
//!     // The `base_context` function can be reused as a starting point
//!     // for pages with more complex requirements.
//!     let mut context = base_context(req.clone());
//!     context.insert("more-info", "data");
//!     tera.render("complex-page.html", &context).unwrap()
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!    let state = web::Data::new(State {
//!        name: "User Name".to_string(),
//!    });
//!
//!    let tera = web::Data::new(match Tera::new("templates/**/*.html") {
//!        Ok(t) => t,
//!        Err(e) => {
//!            println!("Parsing error(s): {}", e);
//!            ::std::process::exit(1);
//!        }
//!    });
//!
//!    HttpServer::new(move || {
//!        App::new()
//!            .app_data(state.clone())
//!            .app_data(tera.clone())
//!            .service(complex_page)
//!            .wrap(TeraPage::new("pages", base_context))
//!    })
//!    .bind(("127.0.0.1", 8080))?
//!    .run()
//!    .await
//! }
//! ```
//!
//! A functional example can be found in the `examples` directory.

use std::future::{ready, Future, Ready};

use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    web::Data,
    Error, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use log::debug;
use tera::{Context, Tera};

/// Middleware constructor.
pub struct TeraPage<C, F>
where
    C: Fn(HttpRequest) -> F,
    F: Future<Output = Context>,
{
    context_builder: C,
    template_prefix: String,
}

impl<C, F> TeraPage<C, F>
where
    C: Fn(HttpRequest) -> F,
    F: Future<Output = Context>,
{
    /// Create a new instance with a given template search prefix and a function that builds the context.
    pub fn new(template_prefix: &str, context_builder: C) -> Self {
        TeraPage {
            context_builder,
            template_prefix: template_prefix.to_string(),
        }
    }
}

impl<S, C, F> Transform<S, ServiceRequest> for TeraPage<C, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
    C: Fn(HttpRequest) -> F + Copy,
    F: Future<Output = Context> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = TeraPageMiddleware<S, C, F>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TeraPageMiddleware {
            service,
            context_builder: self.context_builder,
            template_prefix: self.template_prefix.trim_matches('/').to_string(),
        }))
    }
}

pub struct TeraPageMiddleware<S, C, F>
where
    C: Fn(HttpRequest) -> F,
    F: Future<Output = Context>,
{
    service: S,
    context_builder: C,
    template_prefix: String,
}

impl<S, C, F> Service<ServiceRequest> for TeraPageMiddleware<S, C, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
    C: Fn(HttpRequest) -> F,
    F: Future<Output = Context> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.method() != Method::GET {
            let req = self.service.call(req);
            return Box::pin(async move {
                let r = req.await?;
                Ok(r)
            });
        }

        let tera = if let Some(tera) = req.app_data::<Data<Tera>>() {
            tera.clone()
        } else {
            panic!("A Tera object must be registered as application data for TeraPageMiddlewear to work!");
        };

        let path = req.path().trim_end_matches('/');

        let candidates = if !path.is_empty() {
            vec![
                format!("{}{}.html", self.template_prefix, path),
                format!("{}{}/index.html", self.template_prefix, path),
            ]
        } else {
            vec![format!("{}/index.html", self.template_prefix)]
        };

        debug!("Checking template candidates: {:?}", candidates);

        let templates = tera.get_template_names().collect::<Vec<&str>>();
        let mut matched_template = None;
        for c in candidates {
            if templates.contains(&c.as_str()) {
                matched_template = Some(c);
            }
        }

        if let Some(template) = matched_template {
            debug!("Matched path to template: {:?}", template);
            let context = (self.context_builder)(req.request().clone());

            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Ok().body(tera.render(&template, &context.await).unwrap()),
                ))
            })
        } else {
            debug!("No matching template for path.");
            let req = self.service.call(req);
            Box::pin(async move {
                let r = req.await?;
                Ok(r)
            })
        }
    }
}
