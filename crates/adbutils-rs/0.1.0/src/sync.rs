//! File sync sub-protocol. Port of `adbutils/sync.py`.
//!
//! All message lengths here are **little-endian binary uint32** (unlike the
//! host protocol's ASCII-hex framing). A sync session is entered by switching
//! into device transport and sending `sync:`; each request is
//! `{CMD:4}{u32-le len}{path}`.

use std::path::Path;

use chrono::{DateTime, Local, TimeZone};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::conn::AdbConnection;
use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};
use crate::proto::FileInfo;
use crate::utils::append_path;

const OKAY: &str = "OKAY";
const FAIL: &str = "FAIL";
const DONE: &str = "DONE";
const DATA: &str = "DATA";

// stat mode bit masks (avoid a libc dependency).
const S_IFMT: u32 = 0o170000;
const S_IFDIR: u32 = 0o040000;
const S_IFREG: u32 = 0o100000;

/// Push chunk size (matches Python's 4096; must stay ≤ 64k).
const CHUNK: usize = 4096;

/// Sync client bound to a device.
pub struct Sync {
    device: AdbDevice,
}

impl Sync {
    /// Create a sync client for `device`.
    pub fn new(device: AdbDevice) -> Self {
        Self { device }
    }

    /// Open a sync session and send the `{cmd}{len}{path}` request header
    /// (`_prepare_sync`).
    async fn prepare(&self, path: &str, cmd: &str) -> Result<AdbConnection> {
        let mut c = self.device.open_transport(None, None).await?;
        c.send_command("sync:").await?;
        c.check_okay().await?;
        let path_bytes = path.as_bytes();
        let mut req = Vec::with_capacity(8 + path_bytes.len());
        req.extend_from_slice(cmd.as_bytes());
        req.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
        req.extend_from_slice(path_bytes);
        c.send(&req).await?;
        Ok(c)
    }

    /// `STAT` a path. `mtime == 0` (missing file) yields `mtime: None`.
    pub async fn stat(&self, path: &str) -> Result<FileInfo> {
        let mut c = self.prepare(path, "STAT").await?;
        let tag = c.read_string(4).await?;
        if tag != "STAT" {
            return Err(AdbError::Sync(format!("bad STAT reply: {tag:?}")));
        }
        let buf = c.read_exact(12).await?;
        let mode = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let mtime = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        Ok(FileInfo {
            mode,
            size,
            mtime: to_local(mtime),
            path: path.to_string(),
        })
    }

    /// True if the path exists (`exists`).
    pub async fn exists(&self, path: &str) -> Result<bool> {
        Ok(self.stat(path).await?.mtime.is_some())
    }

    /// `LIST` a directory's immediate entries (`iter_directory` / `list`).
    pub async fn list(&self, path: &str) -> Result<Vec<FileInfo>> {
        let mut c = self.prepare(path, "LIST").await?;
        let mut out = Vec::new();
        loop {
            let response = c.read_string(4).await?;
            if response == DONE {
                break;
            }
            // DENT header: mode, size, mtime, namelen (all u32-le).
            let hdr = c.read_exact(16).await?;
            let mode = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
            let size = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]);
            let mtime = u32::from_le_bytes([hdr[8], hdr[9], hdr[10], hdr[11]]);
            let namelen = u32::from_le_bytes([hdr[12], hdr[13], hdr[14], hdr[15]]) as usize;
            let name = c.read_string(namelen).await?;
            out.push(FileInfo {
                mode,
                size,
                // Python falls back to now() on a bad timestamp; 0 → now here too.
                mtime: to_local(mtime).or_else(|| Some(Local::now())),
                path: name,
            });
        }
        Ok(out)
    }

    /// Push raw bytes to `dst` with `mode` (the `SEND` core). Returns bytes sent.
    pub async fn push_bytes(&self, data: &[u8], dst: &str, mode: u32) -> Result<u64> {
        let path = format!("{dst},{}", S_IFREG | mode);
        let mut c = self.prepare(&path, "SEND").await?;
        let mut total: u64 = 0;
        for chunk in data.chunks(CHUNK) {
            let mut frame = Vec::with_capacity(8 + chunk.len());
            frame.extend_from_slice(DATA.as_bytes());
            frame.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            frame.extend_from_slice(chunk);
            c.send(&frame).await?;
            total += chunk.len() as u64;
        }
        let mtime = Local::now().timestamp() as u32;
        let mut done = Vec::with_capacity(8);
        done.extend_from_slice(DONE.as_bytes());
        done.extend_from_slice(&mtime.to_le_bytes());
        c.send(&done).await?;
        let status = c.read_string(4).await?;
        if status != OKAY {
            return Err(AdbError::Adb(status));
        }
        Ok(total)
    }

    /// Push a local file to `dst`. If `dst` is a directory, the source file name
    /// is appended (mirrors `push`). Returns bytes sent.
    pub async fn push(&self, src: &Path, dst: &str, mode: u32, check: bool) -> Result<u64> {
        let data = tokio::fs::read(src)
            .await
            .map_err(|e| AdbError::Sync(format!("cannot read {}: {e}", src.display())))?;
        let mut dst = dst.to_string();
        // If dst is an existing directory, append the source file name.
        if let Ok(info) = self.stat(&dst).await {
            if info.mode & S_IFDIR != 0 {
                let name = src
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| AdbError::Sync("invalid src file name".into()))?;
                dst = append_path(&dst, name);
            }
        }
        let total = self.push_bytes(&data, &dst, mode).await?;
        if check {
            let file_size = self.stat(&dst).await?.size as u64;
            if total != file_size {
                return Err(AdbError::adb(format!(
                    "Push not complete, expect pushed {total}, actually pushed {file_size}"
                )));
            }
        }
        Ok(total)
    }

    /// Read the full contents of a device file via `RECV` (`read_bytes`).
    pub async fn read_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.recv_into(path, &mut buf).await?;
        Ok(buf)
    }

    /// Read a device file as UTF-8 text (`read_text`).
    pub async fn read_text(&self, path: &str) -> Result<String> {
        let bytes = self.read_bytes(path).await?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// `RECV` a file, streaming DATA chunks into `sink`. Returns bytes written.
    async fn recv_into<W: AsyncWrite + Unpin>(&self, path: &str, sink: &mut W) -> Result<u64> {
        let mut c = self.prepare(path, "RECV").await?;
        let mut total: u64 = 0;
        loop {
            let cmd = c.read_string(4).await?;
            match cmd.as_str() {
                FAIL => {
                    let size = c.read_u32_le().await? as usize;
                    let msg = c.read_string(size).await?;
                    return Err(AdbError::Adb(msg));
                }
                DONE => break,
                DATA => {
                    let size = c.read_u32_le().await? as usize;
                    let chunk = c.read_exact(size).await?;
                    sink.write_all(&chunk)
                        .await
                        .map_err(|e| AdbError::Sync(format!("write failed: {e}")))?;
                    total += size as u64;
                }
                other => return Err(AdbError::Sync(format!("Invalid sync cmd: {other:?}"))),
            }
        }
        Ok(total)
    }

    /// Pull a device file to a local path (`pull_file`). Returns bytes written.
    pub async fn pull_file(&self, src: &str, dst: &Path) -> Result<u64> {
        let mut f = tokio::fs::File::create(dst)
            .await
            .map_err(|e| AdbError::Sync(format!("cannot create {}: {e}", dst.display())))?;
        let n = self.recv_into(src, &mut f).await?;
        f.flush().await.ok();
        Ok(n)
    }

    /// Pull a file or directory from `src` to local `dst` (`pull`).
    pub async fn pull(&self, src: &str, dst: &Path, exist_ok: bool) -> Result<u64> {
        let info = self.stat(src).await?;
        if info.mode & S_IFREG != 0 {
            self.pull_file(src, dst).await
        } else {
            self.pull_dir(src, dst, exist_ok).await
        }
    }

    /// Recursively pull a directory (`pull_dir`). Returns total bytes.
    pub async fn pull_dir(&self, src: &str, dst: &Path, exist_ok: bool) -> Result<u64> {
        // Async recursion → boxed future.
        fn rec<'a>(
            this: &'a Sync,
            src: &'a str,
            dst: &'a Path,
            exist_ok: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + 'a>> {
            Box::pin(async move {
                if let Err(e) = tokio::fs::create_dir_all(dst).await {
                    if !exist_ok {
                        return Err(AdbError::Sync(format!(
                            "cannot create dir {}: {e}",
                            dst.display()
                        )));
                    }
                }
                let mut total: u64 = 0;
                for item in this.list(src).await? {
                    if item.path == "." || item.path == ".." {
                        continue;
                    }
                    let new_src = append_path(src, &item.path);
                    let new_dst = dst.join(&item.path);
                    if item.mode & S_IFMT == S_IFDIR {
                        total += rec(this, &new_src, &new_dst, exist_ok).await?;
                    } else if item.mode & S_IFMT == S_IFREG {
                        total += this.pull_file(&new_src, &new_dst).await?;
                    }
                }
                Ok(total)
            })
        }
        rec(self, src, dst, exist_ok).await
    }
}

/// Convert a unix timestamp to local time, or `None` when zero.
fn to_local(ts: u32) -> Option<DateTime<Local>> {
    if ts == 0 {
        None
    } else {
        Local.timestamp_opt(ts as i64, 0).single()
    }
}

impl AdbDevice {
    /// Sync client for this device (`AdbDevice.sync`).
    pub fn sync(&self) -> Sync {
        Sync::new(self.clone())
    }
}
