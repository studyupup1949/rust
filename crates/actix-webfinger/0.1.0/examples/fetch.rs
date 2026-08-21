use actix::{Actor, System};
use actix_web::client::ClientConnector;
use actix_webfinger::Webfinger;
use futures::Future;
use openssl::ssl::{SslConnector, SslMethod};

fn main() {
    let sys = System::new("sir-boops");

    let ssl_conn = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let conn = ClientConnector::with_connector(ssl_conn).start();

    let fut = Webfinger::fetch(conn, "asonix@asonix.dog", "localhost:8000", false)
        .map(move |w: Webfinger| {
            println!("asonix's webfinger:\n{:#?}", w);

            System::current().stop();
        })
        .map_err(|e| eprintln!("Error: {}", e));

    actix::spawn(fut);

    let _ = sys.run();
}
