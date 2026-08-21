Build instructions
==================

As feature use `serde_impl` or `serde_loose` as you like.


Linux
-----

    cargo build --release --package adif_io --example json2adi --features="serde_impl"
    cargo build --release --package adif_io --example adi2json --features="serde_impl"


### Cross compile for Windows

    sudo apt install mingw-w64
    rustup target add x86_64-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu --release --package adif_io --example json2adi --features="serde_impl"
    cargo build --target x86_64-pc-windows-gnu --release --package adif_io --example adi2json --features="serde_impl"
