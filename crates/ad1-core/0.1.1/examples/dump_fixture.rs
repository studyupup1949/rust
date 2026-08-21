//! Write a spec-faithful sample AD1 image to a path (for fuzz corpus seeds and
//! manual inspection). Requires the `testfix` feature:
//!
//! ```sh
//! cargo run -p ad1-core --features testfix --example dump_fixture -- out.ad1
//! ```
#![allow(clippy::unwrap_used, clippy::expect_used)]

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: dump_fixture <output.ad1>");
    let built = ad1::testfix::build(ad1::testfix::sample_tree());
    std::fs::write(&path, &built.bytes).expect("write image");
    eprintln!("wrote {} bytes to {path}", built.bytes.len());
}
