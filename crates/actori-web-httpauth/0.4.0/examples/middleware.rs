use actori_web::dev::ServiceRequest;
use actori_web::{middleware, web, App, Error, HttpServer};

use actori_web_httpauth::extractors::basic::BasicAuth;
use actori_web_httpauth::middleware::HttpAuthentication;

async fn validator(
    req: ServiceRequest,
    _credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    Ok(req)
}

#[actori_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let auth = HttpAuthentication::basic(validator);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(auth)
            .service(web::resource("/").to(|| async { "Test\r\n" }))
    })
    .bind("127.0.0.1:8080")?
    .workers(1)
    .run()
    .await
}
