use actix::System;
use actix_web::client::Connector;
use actix_webfinger::Webfinger;
use futures::{future::lazy, Future};
use openssl::ssl::{SslConnector, SslMethod};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let sys = System::new("sir-boops");

    let ssl_conn = SslConnector::builder(SslMethod::tls())?.build();
    let conn = Connector::new().ssl(ssl_conn).finish();

    let fut = lazy(move || {
        Webfinger::fetch(conn, "asonix@asonix.dog", "localhost:8000", false)
            .map(move |w: Webfinger| {
                println!("asonix's webfinger:\n{:#?}", w);

                System::current().stop();
            })
            .map_err(|e| eprintln!("Error: {}", e))
    });

    actix::spawn(fut);

    sys.run()?;
    Ok(())
}
