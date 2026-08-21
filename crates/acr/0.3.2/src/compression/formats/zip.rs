use crate::compression::Method;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ZipMethod {
    #[default]
    Stored,
    Shrunk,
    Reduced(u8),
    Imploded,
    Tokenizing,
    Deflated,
    EnhancedDeflated,
    PKWareDCLImploded,
    Reserved(u8),
    Bzip2,
    Lzma,
    Cmpsc,
    Terse,
    Lz77,
    Deprecated,
    Zstd,
    Mp3,
    Xz,
    Jpeg,
    WavPack,
    Ppmd,
    Aes,
    Unsupported,
}

impl From<u16> for ZipMethod {
    fn from(byte: u16) -> Self {
        use ZipMethod::*;
        match byte {
            0 => Stored,
            1 => Shrunk,
            2 => Reduced(1),
            3 => Reduced(2),
            4 => Reduced(3),
            5 => Reduced(4),
            6 => Imploded,
            7 => Tokenizing,
            8 => Deflated,
            9 => EnhancedDeflated,
            10 => PKWareDCLImploded,
            11 => Reserved(1),
            12 => Bzip2,
            13 => Reserved(2),
            14 => Lzma,
            15 => Reserved(3),
            16 => Cmpsc,
            17 => Reserved(4),
            18 => Terse,
            19 => Lz77,
            20 => Deprecated,
            93 => Zstd,
            94 => Mp3,
            95 => Xz,
            96 => Jpeg,
            97 => WavPack,
            98 => Ppmd,
            99 => Aes,
            _ => Unsupported,
        }
    }
}

impl From<ZipMethod> for u16 {
    fn from(method: ZipMethod) -> Self {
        use ZipMethod::*;
        match method {
            Stored => 0,
            Shrunk => 1,
            Reduced(1) => 2,
            Reduced(2) => 3,
            Reduced(3) => 4,
            Reduced(4) => 5,
            Imploded => 6,
            Tokenizing => 7,
            Deflated => 8,
            EnhancedDeflated => 9,
            PKWareDCLImploded => 10,
            Reserved(1) => 11,
            Bzip2 => 12,
            Reserved(2) => 13,
            Lzma => 14,
            Reserved(3) => 15,
            Cmpsc => 16,
            Reserved(4) => 17,
            Terse => 18,
            Lz77 => 19,
            Deprecated => 20,
            Zstd => 93,
            Mp3 => 94,
            Xz => 95,
            Jpeg => 96,
            WavPack => 97,
            Ppmd => 98,
            Aes => 99,
            _ => 0,
        }
    }
}

impl From<ZipMethod> for Method {
    fn from(value: ZipMethod) -> Self {
        use ZipMethod::*;
        match value {
            Stored => Method::None,
            _ => Method::Unsupported,
        }
    }
}
