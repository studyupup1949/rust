# Actix Webpack Proxy
_A simple way to proxy web requests to a webpack dev server with Actix_

### Usage
*This is intended to only be used in Development. For production cases, you should be serving
static assets.*

First, add this project to your dependencies
```toml
# Cargo.toml
actix-webpack-proxy = "0.1"
```

Then, use it in your project:
```rust
// src/main.rs
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
```

### Contributing
Unless otherwise stated, all contributions to this project will be licensed under the CSL with
the exceptions listed in the License section of this file.

### License
This work is licensed under the Cooperative Software License. This is not a Free Software
License, but may be considered a "source-available License." For most hobbyists, self-employed
developers, worker-owned companies, and cooperatives, this software can be used in most
projects so long as this software is distributed under the terms of the CSL. For more
information, see the provided LICENSE file. If none exists, the license can be found online
[here](https://lynnesbian.space/csl/). If you are a free software project and wish to use this
software under the terms of the GNU Affero General Public License, please contact me at
[asonix@asonix.dog](mailto:asonix@asonix.dog) and we can sort that out. If you wish to use this
project under any other license, especially in proprietary software, the answer is likely no.
