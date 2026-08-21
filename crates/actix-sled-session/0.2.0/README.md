# Actix Sled Session
_An Actix Web Session Backend using the Sled embedded database_

- [crates.io](https://crates.io/crates/actix-sled-session)
- [docs.rs](https://docs.rs/actix-sled-session)
- [Join the discussion on Matrix](https://matrix.to/#/!BMSjFeYeYeDoxQxpUr:asonix.dog?via=asonix.dog)

This session backend sets an cookie with a unique ID for each incoming request that doesn't
already have an ID set, and uses that ID as a key to look up session data from a Sled tree.

### Usage
#### Update your Cargo.toml
```toml
[dependencies]
actix = "0.8"
actix-sled-session = "0.2"
actix-web = "1.0"
```

#### Use it in your project
```rust
use actix::System;
use actix_web::{web, App, HttpServer};
use actix_sled_session::{Session, SledSession};

fn index(session: Session) -> String {
    if let Ok(Some(item)) = session.get::<usize>("item") {
        println!("item, {}", item);
        session.clear();
        return format!("Got item, {}", item);
    }

    let _ = session.set::<usize>("item", 3);
    String::from("Set item!")
}

fn main() -> Result<(), failure::Error> {
    let sys = System::new("example");
    let session_backend = SledSession::new_default()?;

    HttpServer::new(move || {
        App::new()
            .wrap(session_backend.clone())
            .route("/", web::get().to(index))
    })
        .bind("127.0.0.1:9876")?
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
