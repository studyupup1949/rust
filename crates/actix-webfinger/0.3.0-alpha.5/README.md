# Actix Webfinger
A library to aid in resolving and providing webfinger objects with the Actix Web web framework.

- [Read the documentation on docs.rs](https://docs.rs/actix-webfinger)
- [Find the crate on crates.io](https://crates.io/crates/actix-webfinger)
- [Hit me up on Mastodon](https://asonix.dog/@asonix)

The main functionality this crate provides is through the `Webfinger::fetch` method for Actix
Web-based clients, and the `Resolver<S>` trait for Actix Web-based servers.

### Usage
First, add Actix Webfinger as a dependency

```toml
[dependencies]
actix = "0.10.0-alpha.1"
actix-web = "3.0.0-alpha.1"
actix-webfinger = "0.3.0-alpha.5"
```

Then use it in your application

#### Client Example
```rust
use actix_web::client::Client;
use actix_webfinger::Webfinger;
use std::error::Error;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::default();
    let wf = Webfinger::fetch(&client, "asonix@asonix.dog", "localhost:8000", false).await?;

    println!("asonix's webfinger:\n{:#?}", wf);
    Ok(())
}
```

#### Server Example
```rust
use actix_web::{web::Data, App, HttpServer};
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
```

### Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

### License

Copyright © 2020 Riley Trautman

Actix Webfinger is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Actix Webfinger is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Tokio ZMQ.

You should have received a copy of the GNU General Public License along with Actix Webfinger. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
