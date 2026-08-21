use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    dev::{Body, BodySize, MessageBody, Payload, ResponseBody, ServiceRequest, ServiceResponse},
    web::{Bytes, BytesMut},
    HttpMessage,
};
use futures::{ready, Stream, StreamExt};
use uuid::Uuid;

struct RequestBufferedMark;
struct ResponseBufferedMark;

pub fn enable_request_buffering<T>(wrapper: T, req: &mut ServiceRequest)
where
    T: AsRef<FileBufferingStreamWrapper>,
{
    if !req.extensions().contains::<RequestBufferedMark>() {
        let inner = req.take_payload();
        req.set_payload(Payload::Stream(wrapper.as_ref().wrap(inner).boxed_local()));

        req.extensions_mut().insert(RequestBufferedMark)
    }
}

pub fn enable_response_buffering<T>(
    wrapper: T,
    mut svc_res: ServiceResponse<Body>,
) -> ServiceResponse<Body>
where
    T: AsRef<FileBufferingStreamWrapper>,
{
    if !svc_res
        .response()
        .extensions()
        .contains::<ResponseBufferedMark>()
    {
        svc_res
            .response_mut()
            .extensions_mut()
            .insert(ResponseBufferedMark);

        svc_res.map_body(|_, rb| {
            let wrapped = wrapper.as_ref().wrap(rb);
            ResponseBody::Body(Body::Message(Box::new(wrapped)))
        })
    } else {
        svc_res
    }
}

// File buffering stream wrapper. After wrap stream can be read multiple times
pub struct FileBufferingStreamWrapper {
    tmp_dir: PathBuf,
    threshold: usize,
    produce_chunk_size: usize,
    buffer_limit: Option<usize>,
}

impl FileBufferingStreamWrapper {
    pub fn new() -> Self {
        Self {
            tmp_dir: std::env::temp_dir(),
            threshold: 1024 * 30,
            produce_chunk_size: 1024 * 30,
            buffer_limit: None,
        }
    }

    // The temporary dir for larger bodies
    pub fn tmp_dir(mut self, v: impl AsRef<Path>) -> Self {
        self.tmp_dir = v.as_ref().to_path_buf();
        self
    }

    // The maximum size in bytes of the in-memory used to buffer the stream. Larger bodies are written to disk
    pub fn threshold(mut self, v: usize) -> Self {
        self.threshold = v;
        self
    }

    // The chunk size for read buffered bodies
    pub fn produce_chunk_size(mut self, v: usize) -> Self {
        self.produce_chunk_size = v;
        self
    }

    // The maximum size in bytes of the body. An attempt to read beyond this limit will cause an error
    pub fn buffer_limit(mut self, v: Option<usize>) -> Self {
        self.buffer_limit = v;
        self
    }

    pub fn wrap<S>(&self, inner: S) -> FileBufferingStream<S> {
        FileBufferingStream::new(
            inner,
            self.tmp_dir.to_path_buf(),
            self.threshold,
            self.produce_chunk_size,
            self.buffer_limit,
        )
    }
}

impl AsRef<FileBufferingStreamWrapper> for FileBufferingStreamWrapper {
    fn as_ref(&self) -> &FileBufferingStreamWrapper {
        self
    }
}

enum Buffer {
    Memory(BytesMut),
    File(PathBuf, File),
}

pub struct FileBufferingStream<S> {
    inner: S,
    inner_eof: bool,

    tmp_dir: PathBuf,
    threshold: usize,
    produce_chunk_size: usize,
    buffer_limit: Option<usize>,

    buffer: Buffer,
    buffer_size: usize,
    produce_index: usize,
}

impl<S> Drop for FileBufferingStream<S> {
    fn drop(&mut self) {
        match self.buffer {
            Buffer::Memory(_) => {}
            Buffer::File(ref path, _) => match std::fs::remove_file(path) {
                Ok(_) => {}
                Err(e) => println!("error at remove buffering file {:?}. {}", path, e),
            },
        };
    }
}

impl<S> FileBufferingStream<S> {
    fn new(
        inner: S,
        tmp_dir: PathBuf,
        threshold: usize,
        produce_chunk_size: usize,
        buffer_limit: Option<usize>,
    ) -> Self {
        Self {
            inner: inner,
            inner_eof: false,

            tmp_dir,
            threshold,
            produce_chunk_size,
            buffer_limit: buffer_limit,

            buffer: Buffer::Memory(BytesMut::new()),
            buffer_size: 0,
            produce_index: 0,
        }
    }

    fn write_to_buffer(&mut self, bytes: &Bytes) -> Result<(), BufferingError> {
        match self.buffer {
            Buffer::Memory(ref mut memory) => {
                if self.threshold < memory.len() + bytes.len() {
                    let mut path = self.tmp_dir.to_path_buf();
                    path.push(Uuid::new_v4().to_simple().to_string());

                    let mut file = OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create_new(true)
                        .open(&path)?;

                    file.write_all(&memory[..])?;
                    file.write_all(bytes)?;

                    self.buffer = Buffer::File(path, file);
                } else {
                    memory.extend_from_slice(bytes)
                }
            }
            Buffer::File(_, ref mut file) => {
                file.write_all(bytes)?;
            }
        }

        self.buffer_size += bytes.len();

        Ok(())
    }

    fn read_from_buffer(&mut self) -> Result<Bytes, BufferingError> {
        let chunk_size = self.produce_chunk_size;
        let buffer_size = self.buffer_size;
        let current_index = self.produce_index;

        if buffer_size <= current_index {
            self.produce_index = 0;
            return Ok(Bytes::new());
        }

        let bytes = match self.buffer {
            Buffer::Memory(ref memory) => {
                let bytes = {
                    if buffer_size <= current_index + chunk_size {
                        self.produce_index = buffer_size;
                        let start = current_index as usize;
                        Bytes::copy_from_slice(&memory[start..])
                    } else {
                        self.produce_index += chunk_size;
                        let start = current_index as usize;
                        let end = (current_index + chunk_size) as usize;
                        Bytes::copy_from_slice(&memory[start..end])
                    }
                };

                bytes
            }
            Buffer::File(_, ref mut file) => {
                if current_index == 0 {
                    file.seek(SeekFrom::Start(0))?;
                    file.flush()?;
                }

                let mut bytes = {
                    if buffer_size <= current_index + chunk_size {
                        self.produce_index = buffer_size;
                        vec![0u8; buffer_size - current_index]
                    } else {
                        self.produce_index += chunk_size;
                        vec![0u8; chunk_size]
                    }
                };

                file.read_exact(bytes.as_mut_slice())?;

                bytes.into()
            }
        };

        Ok(bytes)
    }
}

impl<S, E> FileBufferingStream<S>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
{
    fn generic_poll_next<I>(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, I>>>
    where
        E: Into<I>,
        I: From<BufferingError>,
    {
        let this = self.get_mut();

        match this.inner_eof {
            false => {
                let op = ready!(this.inner.poll_next_unpin(cx));
                match op {
                    Some(ref r) => {
                        if let Ok(ref o) = r {
                            if let Some(limit) = this.buffer_limit {
                                if this.buffer_size + o.len() > limit {
                                    return Poll::Ready(Some(Err(BufferingError::Overflow.into())));
                                }
                            }

                            this.write_to_buffer(o)?;
                        }
                    }
                    None => {
                        this.inner_eof = true;
                    }
                };

                Poll::Ready(op.map(|res| res.map_err(Into::into)))
            }
            true => {
                let bytes = this.read_from_buffer()?;
                if bytes.len() == 0 {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(bytes)))
                }
            }
        }
    }
}

#[derive(Debug)]
enum BufferingError {
    Overflow,
    Io(std::io::Error),
}

impl From<std::io::Error> for BufferingError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl<S, E> MessageBody for FileBufferingStream<S>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
    E: Into<actix_web::Error>,
{    
    fn size(&self) -> BodySize {
        match self.inner_eof {
            false => BodySize::Stream,
            true =>  BodySize::Sized(self.buffer_size as u64)
        }
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, actix_web::Error>>> {
        self.generic_poll_next(cx)
    }
}

impl<S> Stream for FileBufferingStream<S>
where
    S: Stream<Item = Result<Bytes, actix_web::error::PayloadError>> + Unpin,
{
    type Item = Result<Bytes, actix_web::error::PayloadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.generic_poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.inner_eof {
            false => self.inner.size_hint(),
            true => (self.produce_index, Some(self.buffer_size))
        }
    }
}

impl From<BufferingError> for actix_web::error::PayloadError {
    fn from(e: BufferingError) -> Self {
        match e {
            BufferingError::Overflow => actix_web::error::PayloadError::Overflow,
            BufferingError::Io(io) => io.into(),
        }
    }
}

impl From<BufferingError> for actix_web::Error {
    fn from(e: BufferingError) -> Self {
        match e {
            BufferingError::Overflow => actix_web::error::PayloadError::Overflow.into(),
            BufferingError::Io(io) => io.into(),
        }
    }
}
