use actix_web::{middleware, App, HttpServer};
use actix_mark::MarkdownFiles;

const TEMPLATE: &str = include_str!("template.html");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let addr = "127.0.0.1:8080";
    println!("Serving at http://{addr}/docs/");
    println!("  http://{addr}/docs/           → index.md");
    println!("  http://{addr}/docs/about      → about.md");
    println!("  http://{addr}/docs/guide/start → guide/start.md");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                MarkdownFiles::new("/docs", "examples/basic/content")
                    .template(TEMPLATE),
            )
    })
    .bind(addr)?
    .run()
    .await
}
