use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;
use std::time::Duration;

use crate::error::{Error, Result, last_ffi_error};

/// Transport protocol used to reach the Adamo router.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Udp,
    Quic,
    Tcp,
}

impl Protocol {
    fn as_raw(self) -> adamo_sys::adamo_protocol_t {
        match self {
            Protocol::Udp => adamo_sys::ADAMO_PROTOCOL_UDP,
            Protocol::Quic => adamo_sys::ADAMO_PROTOCOL_QUIC,
            Protocol::Tcp => adamo_sys::ADAMO_PROTOCOL_TCP,
        }
    }
}

/// An authenticated Adamo session.
///
/// The underlying libadamo session is reference-counted internally and
/// is safe to use from multiple threads concurrently.
pub struct Session {
    raw: NonNull<adamo_sys::adamo_session_t>,
}

// The C SDK documents the session as thread-safe for concurrent put /
// publisher / subscriber operations.
unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    /// Open a new session authenticated by an API key.
    pub fn open(api_key: &str, protocol: Protocol) -> Result<Self> {
        let key = CString::new(api_key)?;
        // SAFETY: `key` lives for the duration of the call. The C side
        // copies the string; the pointer does not need to outlive it.
        let raw = unsafe { adamo_sys::adamo_open(key.as_ptr(), protocol.as_raw()) };
        match NonNull::new(raw) {
            Some(raw) => Ok(Session { raw }),
            None => Err(last_ffi_error()),
        }
    }

    /// The organisation slug resolved from the API key.
    pub fn org(&self) -> Result<&str> {
        // SAFETY: returns a pointer valid for the lifetime of the session.
        let ptr = unsafe { adamo_sys::adamo_session_org(self.raw.as_ptr()) };
        if ptr.is_null() {
            return Err(last_ffi_error());
        }
        unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .map_err(|_| Error::InvalidUtf8)
    }

    /// Publish a single value.
    pub fn put(&self, key: &str, payload: &[u8], opts: PublishOptions) -> Result<()> {
        let key = CString::new(key)?;
        // SAFETY: all pointers are borrowed for the duration of the call.
        let rc = unsafe {
            adamo_sys::adamo_put(
                self.raw.as_ptr(),
                key.as_ptr(),
                payload.as_ptr(),
                payload.len(),
                opts.priority,
                opts.express as i32,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Declare a persistent publisher on `key`.
    pub fn publisher(&self, key: &str, opts: PublisherOptions) -> Result<Publisher<'_>> {
        let key = CString::new(key)?;
        let raw = unsafe {
            adamo_sys::adamo_publisher(
                self.raw.as_ptr(),
                key.as_ptr(),
                opts.priority,
                opts.express as i32,
                opts.reliable as i32,
            )
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(Publisher {
                raw,
                _session: PhantomData,
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// Publish a log line from the given robot.
    ///
    /// `level` is a free-form string; frontends typically render
    /// `"info" | "warn" | "error" | "debug"` with colour. The payload is
    /// a JSON object `{"ts_us", "level", "message"}` stamped with the
    /// fabric clock so frontends can align log lines across robots.
    pub fn log(&self, name: &str, message: &str, level: &str) -> Result<()> {
        let key = format!("{name}/logs");
        let truncated: String;
        let message_ref = if message.len() > 10_000 {
            let cutoff = message
                .char_indices()
                .take_while(|(i, _)| *i < 10_000)
                .last()
                .map_or(0, |(i, c)| i + c.len_utf8());
            truncated = format!("{}... [truncated]", &message[..cutoff]);
            truncated.as_str()
        } else {
            message
        };
        let payload = format!(
            "{{\"ts_us\":{},\"level\":\"{}\",\"message\":{}}}",
            crate::fabric_now_us(),
            json_escape(level),
            json_quote(message_ref),
        );
        self.put(
            &key,
            payload.as_bytes(),
            PublishOptions {
                priority: 230,
                express: true,
            },
        )
    }

    /// Declare a pull-based subscriber on `key`.
    pub fn subscribe(&self, key: &str) -> Result<Subscriber<'_>> {
        let key = CString::new(key)?;
        let raw = unsafe { adamo_sys::adamo_subscribe(self.raw.as_ptr(), key.as_ptr()) };
        match NonNull::new(raw) {
            Some(raw) => Ok(Subscriber {
                raw,
                _session: PhantomData,
            }),
            None => Err(last_ffi_error()),
        }
    }

}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: handle is non-null and owned.
        unsafe { adamo_sys::adamo_session_free(self.raw.as_ptr()) };
    }
}

/// Escape a string for JSON string-literal embedding (no surrounding quotes).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn json_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    out.push_str(&json_escape(s));
    out.push('"');
    out
}

/// Options for a one-shot `put`.
#[derive(Debug, Clone, Copy)]
pub struct PublishOptions {
    /// Priority 0-255; higher is more important.
    pub priority: u8,
    /// Send as an express (best-effort, bypass congestion control) message.
    pub express: bool,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            priority: 128,
            express: false,
        }
    }
}

/// Options for a persistent publisher.
#[derive(Debug, Clone, Copy)]
pub struct PublisherOptions {
    pub priority: u8,
    pub express: bool,
    pub reliable: bool,
}

impl Default for PublisherOptions {
    fn default() -> Self {
        Self {
            priority: 128,
            express: false,
            reliable: true,
        }
    }
}

/// A declared publisher. Tied to its parent [`Session`] by lifetime.
pub struct Publisher<'a> {
    raw: NonNull<adamo_sys::adamo_publisher_t>,
    _session: PhantomData<&'a Session>,
}

// Publisher mutable state is internal and the C API documents concurrent
// `put` calls as safe; we expose it as `Send + Sync`.
unsafe impl Send for Publisher<'_> {}
unsafe impl Sync for Publisher<'_> {}

impl Publisher<'_> {
    pub fn put(&self, payload: &[u8]) -> Result<()> {
        let rc = unsafe {
            adamo_sys::adamo_publisher_put(self.raw.as_ptr(), payload.as_ptr(), payload.len())
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }
}

impl Drop for Publisher<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_publisher_free(self.raw.as_ptr()) };
    }
}

/// A pull-based subscriber.
pub struct Subscriber<'a> {
    raw: NonNull<adamo_sys::adamo_subscriber_t>,
    _session: PhantomData<&'a Session>,
}

unsafe impl Send for Subscriber<'_> {}

impl Subscriber<'_> {
    /// Block until a sample arrives, or `timeout` elapses (pass `None`
    /// to wait indefinitely).
    pub fn recv(&self, timeout: Option<Duration>) -> Result<Sample> {
        let ms = timeout.map_or(0u64, |d| d.as_millis().min(u64::MAX as u128) as u64);
        let raw = unsafe { adamo_sys::adamo_sub_recv(self.raw.as_ptr(), ms) };
        Sample::from_raw(raw).ok_or_else(|| {
            // NULL can mean either timeout or real error. If libadamo
            // left a message, it's a real error; otherwise it was a
            // timeout.
            match last_ffi_error() {
                Error::Ffi(m) if m == "(no error message)" => Error::Timeout,
                e => e,
            }
        })
    }

    /// Non-blocking receive. Returns `None` when the queue is empty.
    pub fn try_recv(&self) -> Result<Option<Sample>> {
        let raw = unsafe { adamo_sys::adamo_sub_try_recv(self.raw.as_ptr()) };
        if let Some(sample) = Sample::from_raw(raw) {
            return Ok(Some(sample));
        }
        // NULL can be empty-queue or error; disambiguate via last_error.
        match last_ffi_error() {
            Error::Ffi(m) if m == "(no error message)" => Ok(None),
            e => Err(e),
        }
    }
}

impl Drop for Subscriber<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_sub_free(self.raw.as_ptr()) };
    }
}

/// An owned, decoded sample.
#[derive(Debug, Clone)]
pub struct Sample {
    pub key: String,
    pub payload: Vec<u8>,
    pub is_delete: bool,
}

impl Sample {
    /// Take ownership of a raw `adamo_sample_t`. The raw pointer is
    /// freed after the contents are copied out.
    fn from_raw(raw: *mut adamo_sys::adamo_sample_t) -> Option<Self> {
        if raw.is_null() {
            return None;
        }
        // SAFETY: `raw` is non-null and points to a sample owned by us
        // until we call `adamo_sample_free`.
        let sample = unsafe {
            let s = &*raw;
            let key = if s.key.is_null() {
                String::new()
            } else {
                CStr::from_ptr(s.key).to_string_lossy().into_owned()
            };
            let payload = if s.payload.is_null() || s.payload_len == 0 {
                Vec::new()
            } else {
                slice::from_raw_parts(s.payload, s.payload_len).to_vec()
            };
            let is_delete = s.is_delete != 0;
            adamo_sys::adamo_sample_free(raw);
            Sample {
                key,
                payload,
                is_delete,
            }
        };
        Some(sample)
    }
}
