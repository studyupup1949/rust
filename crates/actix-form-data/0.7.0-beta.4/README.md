# Actix Form Data
A library for retrieving form data from Actix Web's multipart streams. It can stream uploaded files
onto the filesystem (its main purpose), but it can also parse associated form data.

- [Read the documentation on docs.rs](https://docs.rs/actix-form-data)
- [Find the crate on crates.io](https://crates.io/crates/actix-form-data)
- [Join the discussion on Matrix](https://matrix.to/#/!YIoRgrEUMubvazlYIH:asonix.dog?via=asonix.dog)

### Usage

Add it to your dependencies.
```toml
# Cargo.toml

[dependencies]
actix-web = "4.0.0"
actix-form-data = "0.6.0"
```

Require it in your project.
```rust
// src/lib.rs or src/main.rs

use form_data::{Field, Form, Value};
```

#### Overview
First, you'd create a form structure you want to parse from the multipart stream.
```rust
let form = Form::<()>::new().field("field-name", Field::text());
```
This creates a form with one required field named "field-name" that will be parsed as text.

Then, pass it as a middleware.
```rust
App::new()
    .wrap(form.clone())
    .service(resource("/upload").route(post().to(upload)))
```

In your handler, get the value

```rust
async fn upload(uploaded_content: Value<()>) -> HttpResponse {
    ...
}
```

And interact with it
```rust
let field_value = match value {
    Value::Map(mut hashmap) => {
        hashmap.remove("field-name")?;
    }
    _ => return None,
};

...
```

#### Example
```rust
/// examples/simple.rs

use actix_form_data::{Error, Field, Form, Value};
use actix_web::{
    web::{post, resource},
    App, HttpResponse, HttpServer,
};
use futures_util::stream::StreamExt;

async fn upload(uploaded_content: Value<()>) -> HttpResponse {
    println!("Uploaded Content: {:#?}", uploaded_content);
    HttpResponse::Created().finish()
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    let form = Form::new()
        .field("Hey", Field::text())
        .field(
            "Hi",
            Field::map()
                .field("One", Field::int())
                .field("Two", Field::float())
                .finalize(),
        )
        .field(
            "files",
            Field::array(Field::file(|_, _, mut stream| async move {
                while let Some(res) = stream.next().await {
                    res?;
                }
                Ok(()) as Result<(), Error>
            })),
        );

    println!("{:?}", form);

    HttpServer::new(move || {
        App::new()
            .wrap(form.clone())
            .service(resource("/upload").route(post().to(upload)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
```

### Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

### License

Copyright © 2021 Riley Trautman

Actix Form Data is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Actix Form Data is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Actix Form Data.

You should have received a copy of the GNU General Public License along with Actix Form Data. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
