# ActivityPub
__This crate defines the base set of types from the ActivityPub specification.__

- [Read the documentation on docs.rs](https://docs.rs/activitypub)
- [Find the crate on crates.io](https://crates.io/crates/activitypub)
- [Hit me up on Mastodon](https://asonix.dog/@asonix)

## Usage
Add the following to your Cargo.toml
```toml
# Cargo.toml

activitypub = "0.4.0-alpha.0"
```

And then use it in your project
```rust
use activitypub::{
    context,
    object::{
        properties::{
            ApObjectProperties,
            ObjectProperties
        },
        Video,
    },
};
use anyhow::Error;

fn init_video_oprops<T>(mut t: T) -> Result<T, Error>
where
    T: AsMut<ObjectProperties>,
{
    // do any object init here
    t.as_mut().set_context_xsd_any_uri(context())?;
    Ok(t)
}

fn init_video_aoprops<T>(mut t: T) -> Result<T, Error>
where
    T: AsMut<ApObjectProperties>,
{
    // do any AP object init here
    t.as_mut().set_likes("https://my-instance.com/likes")?;
    Ok(t)
}

fn main() -> Result<(), Error> {
    let mut video = Video::default();

    init_video_oprops(&mut video)?;
    init_video_aoprops(&mut video)?;

    let video_string = serde_json::to_string(&video)?;

    let video: Video = serde_json::from_str(&video_string)?;

    Ok(())
}
```

## Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

## License

Copyright © 2018 Riley Trautman

ActivityPub is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

ActivityPub is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of ActivityPub.

You should have received a copy of the GNU General Public License along with ActivityPub. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
