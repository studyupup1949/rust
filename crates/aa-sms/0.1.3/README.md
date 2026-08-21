# aa-sms  

[![dependency status][dep-icon]][dep-link]

`aa-sms` is a Rust crate that provides a client for sending messages
with [Andrew & Arnold's SMS API][sms-api].

## Example

This crate is asynchronous and this example uses [Tokio][tokio], so something like this needs to be included in your `Cargo.toml`.

```toml
[dependencies]
tokio = { version = "1.38.0", features = ["full"] }
```

```rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = "+441314960123";
    let password = "hunter2";
    let destination = "+441414960456";
    let message = String::from("⌨ This is a test! 📱");

    let client = aa_sms::Client::builder()
        .username(username)
        .password(password)
        .build();

    let sms = aa_sms::Message::builder()
        .destination(destination)
        .message(message)
        .build();

    client
        .send(sms)
        .await?;

    Ok(())
}

```

## License 

aa-sms - Send messages from Rust with Andrew & Arnold's SMS API

Copyright (C) 2024  Mike Coats

This is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your
option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
General Public License for more details.

[dep-icon]: https://deps.rs/repo/codeberg/MikeCoats/aa-sms/status.svg
[dep-link]: https://deps.rs/repo/codeberg/MikeCoats/aa-sms
[sms-api]: https://support.aa.net.uk/SMS_API
[tokio]: https://tokio.rs/
