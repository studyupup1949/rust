# Actix Webfinger
A library to aid in resolving and providing webfinger objects with the Actix Web web framework.

The main functionality this crate provides is through the `Webfinger::fetch` method for Actix
Web-based clients, and the `Resolver<S>` trait for Actix Web-based servers.

### Usage
First, add Actix Webfinger as a dependency

```toml
[dependencies]
actix = "0.8"
actix-web = "1.0.0-beta.3"
actix-webfinger = "0.2.0-beta.2"
```

Then use it in your application

#### Client Example
```rust
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
```

#### Server Example
```rust
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
```

### Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

### License

Copyright © 2019 Riley Trautman

Actix Webfinger is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Actix Webfinger is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Tokio ZMQ.

You should have received a copy of the GNU General Public License along with Actix Webfinger. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
