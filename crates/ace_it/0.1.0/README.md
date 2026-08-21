# ace_it
### Auto Convert Enums
## Description
Just a smal proc_macro to automatically generate From trait impls for each unnamed variant of an enum

## Usage

Cargo.toml:
```toml
[dependencies]
ace_it = "0.1"
```

## Example
```rs
#[macro_use]
extern crate ace_it;

#[derive(Debug)]
#[ace_it]
enum Error {
  Io(std::io::Error),
  ParseInt(std::num::ParseIntError),
  ParseFloat(std::num::ParseFloatError),
}

use std::io::Read;

fn read_int<R: Read>(reader: &mut R) -> Result<i32, Error> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf.parse()?)
}
```

## Future features
* Attribute for ignoring a variant