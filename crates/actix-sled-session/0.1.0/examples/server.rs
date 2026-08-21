use actix::System;
use actix_session::Session;
use actix_sled_session::SledSession;
use actix_web::{http::header::LOCATION, middleware::Logger, web, App, HttpResponse, HttpServer};
use chrono::{DateTime, Utc};
use failure::Error;

fn handler(session: Session) -> Result<HttpResponse, actix_web::Error> {
    let mut logs = session
        .get::<Vec<DateTime<Utc>>>("access-log")?
        .unwrap_or(Vec::new());
    logs.push(Utc::now());
    session.set("access-log", logs.clone())?;

    Ok(HttpResponse::Ok().json(logs))
}

fn clear(session: Session) -> HttpResponse {
    session.clear();
    HttpResponse::SeeOther().header(LOCATION, "/").finish()
}

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = System::new("example-server");

    let session = SledSession::new_default()?;

    HttpServer::new(move || {
        App::new()
            .wrap(session.clone())
            .wrap(Logger::default())
            .route("/", web::get().to(handler))
            .route("/clear", web::get().to(clear))
    })
    .bind("127.0.0.1:8079")?
    .start();

    sys.run()?;

    Ok(())
}
