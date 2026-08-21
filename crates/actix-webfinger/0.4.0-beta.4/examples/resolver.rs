use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use actix_webfinger::{Resolver, Webfinger};
use std::{error::Error, future::Future, pin::Pin};

#[derive(Clone, Debug)]
pub struct MyState {
    domain: String,
}

pub struct MyResolver;

impl Resolver for MyResolver {
    type State = Data<MyState>;
    type Error = actix_web::error::JsonPayloadError;

    fn find(
        scheme: Option<&str>,
        account: &str,
        domain: &str,
        state: Data<MyState>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Webfinger>, Self::Error>>>> {
        let w = if scheme == Some("acct:") && domain == state.domain {
            Some(Webfinger::new(&format!("{}@{}", account, domain)))
        } else {
            None
        };

        Box::pin(async move { Ok(w) })
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(MyState {
                domain: "localhost:8000".to_owned(),
            }))
            .wrap(Logger::default())
            .service(actix_webfinger::resource::<MyResolver>())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await?;

    Ok(())
}
