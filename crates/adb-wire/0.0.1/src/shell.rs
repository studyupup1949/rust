//! Shell protocol (v2) stream types and packet demuxing.

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;

use crate::error::Result;

// Shell v2 packets:
//   [1 byte id] [4 byte LE length] [payload]
//
// Stream IDs:
//   0 = stdin  (host → device)
//   1 = stdout (device → host)
//   2 = stderr (device → host)
//   3 = exit   (device → host, 1-byte payload = exit code)
//   4 = close stdin (device → host, signals stdin closed)

const ID_STDIN: u8 = 0;
const ID_STDOUT: u8 = 1;
const ID_STDERR: u8 = 2;
const ID_EXIT: u8 = 3;
const ID_CLOSE_STDIN: u8 = 4;

/// Maximum payload size for a single shell packet (16 MiB).
const MAX_SHELL_PAYLOAD: usize = 16 * 1024 * 1024;

/// Output from a shell command, with separated stdout/stderr and exit code.
#[derive(Debug, Clone)]
pub struct ShellOutput {
    /// Standard output from the command.
    pub stdout: Vec<u8>,
    /// Standard error from the command.
    pub stderr: Vec<u8>,
    /// Exit code of the command (0 = success).
    pub exit_code: u8,
}

impl ShellOutput {
    /// Get stdout as a trimmed UTF-8 string.
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).trim().to_string()
    }

    /// Get stderr as a trimmed UTF-8 string.
    pub fn stderr_str(&self) -> String {
        String::from_utf8_lossy(&self.stderr).trim().to_string()
    }

    /// Returns `true` if the command exited with code 0.
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Streaming reader for shell command output.
///
/// Demultiplexes stdout, stderr, and exit code from the shell v2 packet
/// framing. Implements [`AsyncRead`] which yields only stdout bytes.
/// After the stream is fully consumed, call [`exit_code`](Self::exit_code)
/// and [`stderr`](Self::stderr) to inspect the results.
pub struct ShellStream {
    inner: TcpStream,
    stdout_buf: Vec<u8>,
    stdout_pos: usize,
    stderr: Vec<u8>,
    exit_code: Option<u8>,
    done: bool,
    header_buf: [u8; 5],
    header_pos: usize,
    payload_buf: Vec<u8>,
    payload_pos: usize,
}

impl ShellStream {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            inner: stream,
            stdout_buf: Vec::new(),
            stdout_pos: 0,
            stderr: Vec::new(),
            exit_code: None,
            done: false,
            header_buf: [0u8; 5],
            header_pos: 0,
            payload_buf: Vec::new(),
            payload_pos: 0,
        }
    }

    /// Consume the stream, collecting all output.
    pub async fn collect_output(mut self) -> Result<ShellOutput> {
        let mut stdout = Vec::new();
        loop {
            let more = self.read_next_packet().await?;
            if self.stdout_pos < self.stdout_buf.len() {
                stdout.extend_from_slice(&self.stdout_buf[self.stdout_pos..]);
                self.stdout_buf.clear();
                self.stdout_pos = 0;
            }
            if !more {
                break;
            }
        }
        Ok(ShellOutput {
            stdout,
            stderr: self.stderr,
            exit_code: self.exit_code.unwrap_or(255),
        })
    }

    /// Get accumulated stderr bytes (available as data arrives).
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    /// Get the exit code, if received.
    pub fn exit_code(&self) -> Option<u8> {
        self.exit_code
    }

    /// Access the underlying [`TcpStream`].
    pub fn as_tcp_stream(&self) -> &TcpStream {
        &self.inner
    }

    /// Send data to the command's stdin.
    pub async fn write_stdin(&mut self, data: &[u8]) -> Result<()> {
        let mut pkt = Vec::with_capacity(5 + data.len());
        pkt.push(ID_STDIN);
        pkt.extend_from_slice(&(data.len() as u32).to_le_bytes());
        pkt.extend_from_slice(data);
        self.inner.write_all(&pkt).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// Close the command's stdin.
    ///
    /// Signals EOF to the remote process, causing reads from stdin to
    /// return zero. Required for commands that read until EOF (e.g. `cat`,
    /// `base64 -d`) to terminate.
    pub async fn close_stdin(&mut self) -> Result<()> {
        let pkt: [u8; 5] = [ID_CLOSE_STDIN, 0, 0, 0, 0];
        self.inner.write_all(&pkt).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// Read the next packet, buffering stdout and stderr.
    /// Returns true if more data may follow, false on exit/EOF.
    async fn read_next_packet(&mut self) -> io::Result<bool> {
        let mut header = [0u8; 5];
        match self.inner.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                self.done = true;
                return Ok(false);
            }
            Err(e) => return Err(e),
        }

        let id = header[0];
        let len = u32::from_le_bytes(header[1..5].try_into().unwrap()) as usize;

        if len > MAX_SHELL_PAYLOAD {
            self.done = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("shell payload too large: {len} bytes"),
            ));
        }

        let mut payload = vec![0u8; len];
        if len > 0 {
            self.inner.read_exact(&mut payload).await?;
        }

        match id {
            ID_STDOUT => self.stdout_buf.extend_from_slice(&payload),
            ID_STDERR => self.stderr.extend_from_slice(&payload),
            ID_EXIT => {
                self.exit_code = payload.first().copied();
                self.done = true;
                return Ok(false);
            }
            _ => {}
        }

        Ok(true)
    }
}

impl AsyncRead for ShellStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        loop {
            if this.stdout_pos < this.stdout_buf.len() {
                let available = &this.stdout_buf[this.stdout_pos..];
                let n = available.len().min(buf.remaining());
                buf.put_slice(&available[..n]);
                this.stdout_pos += n;
                if this.stdout_pos == this.stdout_buf.len() {
                    this.stdout_buf.clear();
                    this.stdout_pos = 0;
                }
                return Poll::Ready(Ok(()));
            }

            if this.done {
                return Poll::Ready(Ok(()));
            }

            while this.header_pos < 5 {
                let mut tmp = ReadBuf::new(&mut this.header_buf[this.header_pos..]);
                match Pin::new(&mut this.inner).poll_read(cx, &mut tmp) {
                    Poll::Ready(Ok(())) => {
                        let n = tmp.filled().len();
                        if n == 0 {
                            this.done = true;
                            return Poll::Ready(Ok(()));
                        }
                        this.header_pos += n;
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => return Poll::Pending,
                }
            }

            if this.payload_buf.is_empty() && this.payload_pos == 0 {
                let len = u32::from_le_bytes(
                    this.header_buf[1..5].try_into().unwrap(),
                ) as usize;

                if len > MAX_SHELL_PAYLOAD {
                    this.done = true;
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("shell payload too large: {len} bytes"),
                    )));
                }

                if len > 0 {
                    this.payload_buf.resize(len, 0);
                }
            }

            while this.payload_pos < this.payload_buf.len() {
                let mut tmp =
                    ReadBuf::new(&mut this.payload_buf[this.payload_pos..]);
                match Pin::new(&mut this.inner).poll_read(cx, &mut tmp) {
                    Poll::Ready(Ok(())) => {
                        let n = tmp.filled().len();
                        if n == 0 {
                            this.done = true;
                            return Poll::Ready(Ok(()));
                        }
                        this.payload_pos += n;
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => return Poll::Pending,
                }
            }

            let id = this.header_buf[0];
            let payload = std::mem::take(&mut this.payload_buf);
            this.header_pos = 0;
            this.payload_pos = 0;

            match id {
                ID_STDOUT => {
                    this.stdout_buf = payload;
                    this.stdout_pos = 0;
                }
                ID_STDERR => this.stderr.extend_from_slice(&payload),
                ID_EXIT => {
                    this.exit_code = payload.first().copied();
                    this.done = true;
                    return Poll::Ready(Ok(()));
                }
                _ => {}
            }
        }
    }
}

/// Read all shell packets from a stream, returning the collected output.
pub(crate) async fn read_shell(stream: TcpStream) -> Result<ShellOutput> {
    ShellStream::new(stream).collect_output().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_output_methods() {
        let out = ShellOutput {
            stdout: b"  hello\n".to_vec(),
            stderr: b"warn\n".to_vec(),
            exit_code: 0,
        };
        assert_eq!(out.stdout_str(), "hello");
        assert_eq!(out.stderr_str(), "warn");
        assert!(out.success());
    }

    #[test]
    fn shell_output_failure() {
        let out = ShellOutput {
            stdout: Vec::new(),
            stderr: b"error".to_vec(),
            exit_code: 1,
        };
        assert!(!out.success());
    }
}
