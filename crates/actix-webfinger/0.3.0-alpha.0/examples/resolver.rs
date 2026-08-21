use actix_web::{web::Data, App, HttpServer};
use actix_webfinger::{Resolver, Webfinger};
use std::{error::Error, future::Future, pin::Pin};

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
    ) -> Pin<Box<dyn Future<Output = Result<Option<Webfinger>, Self::Error>>>> {
        let w = if domain == state.domain {
            Some(Webfinger::new(&format!("{}@{}", account, domain)))
        } else {
            None
        };

        Box::pin(async move { Ok(w) })
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    HttpServer::new(|| {
        App::new()
            .data(MyState {
                domain: "asonix.dog".to_owned(),
            })
            .service(actix_webfinger::resource::<_, MyResolver>())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await?;

    Ok(())
}
