use actix::System;
use actix_web::{client::Client, App, HttpServer};
use actix_webpack_proxy::{default_route, ws_resource, DefaultProxy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sys = System::new("dev-system");

    HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(DefaultProxy)
            .service(ws_resource::<DefaultProxy>())
            .default_service(default_route::<DefaultProxy>())
    })
    .bind("0.0.0.0:8080")?
    .start();

    sys.run()?;

    Ok(())
}
