//! On-disk `.acc` binary format.
//!
//! ```text
//! +----------------------------------+
//! | Magic       "ACC1"   (4 bytes)   |
//! | Version     u16 (LE)             |
//! | Flags       u16 (LE)             |   bit 0: zstd-compressed payload
//! | Count       u32 (LE)             |
//! | Reserved    [u8; 4]              |
//! +----------------------------------+
//! | Payload (optionally zstd-framed) |
//! |   bincode-serialized Vec<Entry>  |
//! +----------------------------------+
//! ```

mod compression;
mod entry;
mod header;
mod reader;
mod writer;

pub use entry::Entry;
pub use header::{FLAG_ZSTD, FORMAT_VERSION, HEADER_SIZE, Header, MAGIC};
pub use reader::{AccReader, read_file};
pub use writer::{AccWriter, write_file};
