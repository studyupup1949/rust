# abootimg-oxide

[![Crates.io](https://img.shields.io/crates/v/abootimg-oxide)](https://lib.rs/crates/abootimg-oxide)
[![Documentation](https://docs.rs/abootimg-oxide/badge.svg)](https://docs.rs/abootimg-oxide)
![Crates.io MSRV](https://img.shields.io/crates/msrv/abootimg-oxide)
![Crates.io License](https://img.shields.io/crates/l/abootimg-oxide)
![Crates.io Total Downloads](https://img.shields.io/crates/d/abootimg-oxide)

Android boot image (boot.img) parser written in Rust

`unpack_bootimg` has been fully reimplemented using this library. Try it by
running `cargo run --package=unpack_bootimg -- --boot_img <path to boot.img>`.

TODO: reimplement mkbootimg

## Examples

```rs
use std::fs::File;
use abootimg_oxide::{BufReader, Header};

let mut r = BufReader::new(File::open("boot_a.img").unwrap());
let hdr = Header::parse(&mut r).unwrap();
println!("{hdr:#?}");

// Extract the kernel
use std::io::{self, BufWriter, Read, Seek, SeekFrom};

let mut w = BufWriter::new(File::create("boot_a_kernel").unwrap());
let r = r.get_mut();
r.seek(SeekFrom::Start(hdr.kernel_position() as u64))
    .unwrap();
io::copy(&mut r.take(hdr.kernel_size().into()), w.get_mut()).unwrap();
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
