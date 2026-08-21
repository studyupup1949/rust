use std::{convert::Infallible, io, time::Duration};

use actix_web::{
    get,
    middleware::{Logger, TrailingSlash},
    App, HttpRequest, HttpServer, Responder,
};
use actix_web_lab::{extract::Path, middleware::NormalizePath, respond::Html, sse};
use futures_util::stream;
use time::format_description::well_known::Rfc3339;
use tokio::time::sleep;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    tracing::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Always).use_redirects())
            .wrap(Logger::default())
            .service(
                actix_files::Files::new("/", "/actix-web-lab/examples")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
