pub mod algorithms;
pub mod compressor;
pub mod decompressor;

mod formats;

pub use formats::hssp::Hssp4Method;
pub use formats::zip::ZipMethod;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Method {
    #[default]
    None,
    Deflate,
    Lzma,
    Unsupported,
}
