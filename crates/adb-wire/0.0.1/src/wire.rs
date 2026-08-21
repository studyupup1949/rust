//! Hex-length framed ADB wire protocol helpers.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{Error, Result};

/// Maximum response size from a hex-length-prefixed message (1 MiB).
const MAX_HEX_LEN_BYTES: usize = 1024 * 1024;

pub async fn send(w: &mut (impl AsyncWriteExt + Unpin), payload: &str) -> Result<()> {
    let msg = format!("{:04x}{payload}", payload.len());
    w.write_all(msg.as_bytes()).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_okay(r: &mut (impl AsyncReadExt + Unpin)) -> Result<()> {
    let mut status = [0u8; 4];
    r.read_exact(&mut status).await?;

    match &status {
        b"OKAY" => Ok(()),
        b"FAIL" => {
            let len = read_hex_len(r).await?.min(256 * 1024);
            let mut msg = vec![0u8; len];
            r.read_exact(&mut msg).await?;
            Err(Error::Adb(String::from_utf8_lossy(&msg).to_string()))
        }
        _ => Err(Error::Protocol(format!(
            "expected OKAY/FAIL, got {:?}",
            String::from_utf8_lossy(&status)
        ))),
    }
}

pub async fn read_hex_len(r: &mut (impl AsyncReadExt + Unpin)) -> Result<usize> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).await?;
    let s =
        std::str::from_utf8(&buf).map_err(|_| Error::Protocol("invalid hex length".into()))?;
    let len = usize::from_str_radix(s, 16)
        .map_err(|_| Error::Protocol(format!("bad hex length: {s}")))?;
    if len > MAX_HEX_LEN_BYTES {
        return Err(Error::Protocol(format!(
            "response too large: {len} bytes (max {MAX_HEX_LEN_BYTES})"
        )));
    }
    Ok(len)
}
