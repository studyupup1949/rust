use actix_web::{web::Data, App, HttpServer};
use actix_webfinger::{Resolver, Webfinger};
use futures::{future::IntoFuture, Future};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct MyState {
    domain: String,
}

pub struct MyResolver;

impl Resolver<Data<MyState>> for MyResolver {
    type Error = actix_web::error::JsonPayloadError;

    fn find(
        account: &str,
        domain: &str,
        state: &Data<MyState>,
    ) -> Box<dyn Future<Item = Option<Webfinger>, Error = Self::Error>> {
        let w = if domain == state.domain {
            Some(Webfinger::new(&format!("{}@{}", account, domain)))
        } else {
            None
        };

        Box::new(Ok(w).into_future())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    HttpServer::new(|| {
        App::new()
            .data(MyState {
                domain: "asonix.dog".to_owned(),
            })
            .service(actix_webfinger::resource::<_, MyResolver>())
    })
    .bind("127.0.0.1:8000")?
    .run()?;

    Ok(())
}
