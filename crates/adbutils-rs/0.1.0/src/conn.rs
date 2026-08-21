//! Async smartsocket connection. Port of `AdbConnection` in `adbutils/_adb.py`.
//!
//! Framing primitives for the **host / local-service** protocol: a 4-byte ASCII
//! lowercase-hex length prefix followed by the UTF-8 payload, with `OKAY`/`FAIL`
//! status replies. The sync sub-protocol (little-endian binary lengths) is built
//! on top of the raw [`AdbConnection::send`] / [`AdbConnection::read`] helpers in
//! `sync.rs`.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::errors::{AdbError, Result};

const OKAY: &[u8; 4] = b"OKAY";
const FAIL: &[u8; 4] = b"FAIL";

/// Timeout used only for the initial TCP connect (matches Python's `settimeout(3)`).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// A single smartsocket connection to the adb server. Host commands use one
/// connection each; local services (`shell:`, `sync:`, …) switch the socket into
/// device context and then consume it.
pub struct AdbConnection {
    stream: Option<TcpStream>,
    /// Optional per-read timeout (Python's `socket_timeout`). `None` = block.
    timeout: Option<Duration>,
}

impl AdbConnection {
    /// Connect to the adb server at `host:port`. On failure, runs
    /// `adb start-server` once and retries (mirrors `_safe_connect`).
    pub async fn connect(host: &str, port: u16, timeout: Option<Duration>) -> Result<Self> {
        match Self::create_socket(host, port).await {
            Ok(stream) => Ok(Self { stream: Some(stream), timeout }),
            Err(_) => {
                crate::utils::start_server().await?;
                let stream = Self::create_socket(host, port).await?;
                Ok(Self { stream: Some(stream), timeout })
            }
        }
    }

    async fn create_socket(host: &str, port: u16) -> Result<TcpStream> {
        let fut = TcpStream::connect((host, port));
        let stream = match tokio::time::timeout(CONNECT_TIMEOUT, fut).await {
            Err(_) => return Err(AdbError::Timeout("connect to adb server timeout".into())),
            Ok(Err(e)) => {
                return Err(AdbError::Connection(format!(
                    "connect to adb server failed: {e}"
                )))
            }
            Ok(Ok(s)) => s,
        };
        let _ = stream.set_nodelay(true);
        Ok(stream)
    }

    /// Set the per-read timeout.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    fn stream(&mut self) -> Result<&mut TcpStream> {
        self.stream
            .as_mut()
            .ok_or_else(|| AdbError::adb("Connection is closed"))
    }

    /// True once the connection has been closed.
    pub fn closed(&self) -> bool {
        self.stream.is_none()
    }

    /// Close the write side then drop the socket (mirrors `close`).
    pub async fn close(&mut self) {
        if let Some(mut s) = self.stream.take() {
            let _ = s.shutdown().await;
        }
    }

    /// Take ownership of the raw stream, consuming the wrapper. Used by
    /// `create_connection`, which hands the transparent pipe to the caller.
    pub fn into_stream(mut self) -> Result<TcpStream> {
        self.stream
            .take()
            .ok_or_else(|| AdbError::adb("Connection is closed"))
    }

    /// Write raw bytes (alias of `conn.send`).
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.stream()?;
        stream.write_all(data).await?;
        Ok(())
    }

    /// Read up to `n` bytes, returning fewer only on EOF (`_read_fully`/`read`).
    pub async fn read(&mut self, n: usize) -> Result<Vec<u8>> {
        let timeout = self.timeout;
        let stream = self.stream()?;
        let mut buf = vec![0u8; n];
        let mut filled = 0;
        while filled < n {
            let read_fut = stream.read(&mut buf[filled..]);
            let got = match timeout {
                Some(d) => match tokio::time::timeout(d, read_fut).await {
                    Err(_) => return Err(AdbError::Timeout("adb read timeout".into())),
                    Ok(r) => r?,
                },
                None => read_fut.await?,
            };
            if got == 0 {
                break; // EOF
            }
            filled += got;
        }
        buf.truncate(filled);
        Ok(buf)
    }

    /// Read exactly `n` bytes or error (`read_exact`).
    pub async fn read_exact(&mut self, n: usize) -> Result<Vec<u8>> {
        let data = self.read(n).await?;
        if data.len() < n {
            return Err(AdbError::Connection(format!(
                "Expected {n} bytes, got {}",
                data.len()
            )));
        }
        Ok(data)
    }

    /// Read a little-endian u32 (`read_uint32`, used by framebuffer).
    pub async fn read_u32_le(&mut self) -> Result<u32> {
        let data = self.read_exact(4).await?;
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Send a host/local-service command: 4-hex length prefix + UTF-8 payload
    /// (`send_command`).
    pub async fn send_command(&mut self, cmd: &str) -> Result<()> {
        let bytes = cmd.as_bytes();
        let header = format!("{:04x}", bytes.len());
        let mut packet = Vec::with_capacity(4 + bytes.len());
        packet.extend_from_slice(header.as_bytes());
        packet.extend_from_slice(bytes);
        self.send(&packet).await
    }

    /// Read `n` bytes and decode as UTF-8 (lossy, like Python `errors="replace"`).
    pub async fn read_string(&mut self, n: usize) -> Result<String> {
        let data = self.read(n).await?;
        Ok(String::from_utf8_lossy(&data).into_owned())
    }

    /// Read a length-prefixed reply block: 4 hex chars length then that many
    /// bytes (`read_string_block`).
    pub async fn read_string_block(&mut self) -> Result<String> {
        let length = self.read_string(4).await?;
        if length.is_empty() {
            return Err(AdbError::adb("connection closed"));
        }
        let size = usize::from_str_radix(length.trim(), 16)
            .map_err(|_| AdbError::adb(format!("invalid length block: {length:?}")))?;
        self.read_string(size).await
    }

    /// Read all remaining bytes until the peer closes (`read_until_close`).
    pub async fn read_until_close_bytes(&mut self) -> Result<Vec<u8>> {
        let mut content = Vec::new();
        loop {
            let chunk = self.read(4096).await?;
            if chunk.is_empty() {
                break;
            }
            content.extend_from_slice(&chunk);
        }
        Ok(content)
    }

    /// Read all remaining bytes and decode lossily as UTF-8.
    pub async fn read_until_close(&mut self) -> Result<String> {
        let bytes = self.read_until_close_bytes().await?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Read the 4-byte status; `OKAY` → ok, `FAIL` → read and raise the error
    /// block (`check_okay`).
    pub async fn check_okay(&mut self) -> Result<()> {
        let data = self.read(4).await?;
        if data == FAIL {
            let msg = self.read_string_block().await?;
            return Err(AdbError::Adb(msg));
        }
        if data == OKAY {
            return Ok(());
        }
        Err(AdbError::adb(format!("Unknown data: {data:?}")))
    }
}
