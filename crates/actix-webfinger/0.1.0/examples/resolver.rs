use actix_web::server;
use actix_webfinger::{Resolver, Webfinger};
use futures::{future::IntoFuture, Future};

#[derive(Clone, Debug)]
pub struct MyState {
    domain: String,
}

pub struct MyResolver;

impl Resolver<MyState> for MyResolver {
    type Error = actix_web::error::JsonPayloadError;

    fn find(
        account: &str,
        domain: &str,
        state: &MyState,
    ) -> Box<dyn Future<Item = Option<Webfinger>, Error = Self::Error>> {
        let w = if domain == state.domain {
            Some(Webfinger::new(&format!("{}@{}", account, domain)))
        } else {
            None
        };

        Box::new(Ok(w).into_future())
    }
}

fn main() {
    server::new(|| {
        actix_webfinger::app::<MyState, MyResolver>(MyState {
            domain: "asonix.dog".to_owned(),
        })
    })
    .bind("127.0.0.1:8000")
    .unwrap()
    .run();
}
