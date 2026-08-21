use actori_redis::RedisSession;
use actori_session::Session;
use actori_web::{middleware, web, App, Error, HttpRequest, HttpServer, Responder};

/// simple handler
async fn index(req: HttpRequest, session: Session) -> Result<impl Responder, Error> {
    println!("{:?}", req);

    // session
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        session.set("counter", count + 1)?;
    } else {
        session.set("counter", 1)?;
    }

    Ok("Welcome!")
}

#[actori_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actori_web=info,actori_redis=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // cookie session middleware
            .wrap(RedisSession::new("127.0.0.1:6379", &[0; 32]))
            // register simple route, handle all methods
            .service(web::resource("/").to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
