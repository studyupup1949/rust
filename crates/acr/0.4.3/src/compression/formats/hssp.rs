use crate::compression::Method;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Hssp4Method {
    #[default]
    None,
    Lzma,
    Deflate,
    DeflateZlib,
    Unsupported,
}

impl From<[u8; 4]> for Hssp4Method {
    fn from(bytes: [u8; 4]) -> Self {
        use Hssp4Method::*;
        match &bytes {
            b"NONE" => None,
            b"LZMA" => Lzma,
            b"DEFL" => Deflate,
            b"DFLT" => DeflateZlib,
            _ => Unsupported,
        }
    }
}

impl From<Hssp4Method> for [u8; 4] {
    fn from(method: Hssp4Method) -> [u8; 4] {
        use Hssp4Method::*;
        match method {
            None => *b"NONE",
            Lzma => *b"LZMA",
            DeflateZlib => *b"DEFL",
            Deflate => *b"DFLT",
            _ => *b"\0\0\0\0",
        }
    }
}

impl From<Method> for Hssp4Method {
    fn from(method: Method) -> Self {
        use Hssp4Method::*;
        match method {
            Method::None => None,
            Method::Lzma => Lzma,
            Method::Deflate => Deflate,
            Method::DeflateZlib => DeflateZlib,
            _ => Unsupported,
        }
    }
}

impl From<Hssp4Method> for Method {
    fn from(method: Hssp4Method) -> Self {
        use Hssp4Method::*;
        match method {
            None => Method::None,
            Lzma => Method::Lzma,
            Deflate => Method::Deflate,
            DeflateZlib => Method::DeflateZlib,
            _ => Method::Unsupported,
        }
    }
}
