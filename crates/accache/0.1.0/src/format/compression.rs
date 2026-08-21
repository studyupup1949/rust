use std::io::{self, Read, Write};

use zstd::stream::{Decoder, Encoder};

use crate::config::Compression;

pub enum CompressedWriter<W: Write> {
    Plain(W),
    Zstd(Encoder<'static, W>),
}

impl<W: Write> CompressedWriter<W> {
    pub fn new(inner: W, compression: Compression) -> io::Result<Self> {
        match compression {
            Compression::None => Ok(Self::Plain(inner)),
            Compression::Zstd { level } => Ok(Self::Zstd(Encoder::new(inner, level)?)),
        }
    }

    /// finalize the underlying compressor and recover the writer
    pub fn finish(self) -> io::Result<W> {
        match self {
            Self::Plain(w) => Ok(w),
            Self::Zstd(enc) => enc.finish(),
        }
    }
}

impl<W: Write> Write for CompressedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            Self::Zstd(z) => z.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Zstd(z) => z.flush(),
        }
    }
}

pub enum CompressedReader<R: Read> {
    Plain(R),
    Zstd(Decoder<'static, std::io::BufReader<R>>),
}

impl<R: Read> CompressedReader<R> {
    pub fn new(inner: R, compressed: bool) -> io::Result<Self> {
        if compressed {
            Ok(Self::Zstd(Decoder::new(inner)?))
        } else {
            Ok(Self::Plain(inner))
        }
    }
}

impl<R: Read> Read for CompressedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(r) => r.read(buf),
            Self::Zstd(z) => z.read(buf),
        }
    }
}
