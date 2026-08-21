use std::ffi::{CStr, CString, c_char, c_void};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::slice;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::{Error, Result, last_ffi_error};

/// Pose publishes are throttled so a tight odometry loop can call
/// [`Session::set_pose`] unconditionally without flooding the control plane.
const POSE_MIN_INTERVAL: Duration = Duration::from_millis(200);

/// Transport protocol used to reach the Adamo router.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Udp,
    Quic,
    Tcp,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Quic
    }
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
    pose_last_pub: Mutex<Option<Instant>>,
}

// The C SDK documents the session as thread-safe for concurrent put /
// publisher / subscriber operations.
unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    pub(crate) fn raw_ptr(&self) -> *const adamo_sys::adamo_session_t {
        self.raw.as_ptr()
    }

    /// Open a new session authenticated by an API key.
    pub fn open(api_key: &str, protocol: Protocol) -> Result<Self> {
        let key = CString::new(api_key)?;
        // SAFETY: `key` lives for the duration of the call. The C side
        // copies the string; the pointer does not need to outlive it.
        let raw = unsafe { adamo_sys::adamo_open(key.as_ptr(), protocol.as_raw()) };
        match NonNull::new(raw) {
            Some(raw) => Ok(Session {
                raw,
                pose_last_pub: Mutex::new(None),
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// Open a new mTLS-authenticated session using certificate material from
    /// the environment/configuration understood by the native SDK.
    pub fn open_mtls(api_key: &str, protocol: Protocol) -> Result<Self> {
        let key = CString::new(api_key)?;
        let raw = unsafe { adamo_sys::adamo_open_mtls(key.as_ptr(), protocol.as_raw()) };
        match NonNull::new(raw) {
            Some(raw) => Ok(Session {
                raw,
                pose_last_pub: Mutex::new(None),
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// Open a new session authenticated by an API key using the default
    /// transport.
    pub fn open_default(api_key: &str) -> Result<Self> {
        let key = CString::new(api_key)?;
        let raw = unsafe { adamo_sys::adamo_open_default(key.as_ptr()) };
        match NonNull::new(raw) {
            Some(raw) => Ok(Session {
                raw,
                pose_last_pub: Mutex::new(None),
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// The organisation slug resolved from the API key.
    pub fn org(&self) -> Result<&str> {
        // SAFETY: the C API returns a non-NUL-terminated pointer plus length,
        // both valid for the lifetime of the session.
        let ptr = unsafe { adamo_sys::adamo_session_org(self.raw.as_ptr()) };
        if ptr.is_null() {
            return Err(last_ffi_error());
        }
        let len = unsafe { adamo_sys::adamo_session_org_len(self.raw.as_ptr()) };
        let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len) };
        std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
    }

    /// Most recent best RTT to the connected relay's time plugin.
    /// Returns `Ok(None)` until the time-sync loop has received a pong.
    pub fn relay_rtt(&self) -> Result<Option<Duration>> {
        let mut out_us: u64 = 0;
        let rc = unsafe { adamo_sys::adamo_relay_rtt_us(self.raw.as_ptr(), &mut out_us) };
        if rc == 0 {
            return Ok(Some(Duration::from_micros(out_us)));
        }
        match last_ffi_error() {
            Error::Ffi(m) if m.contains("not available yet") => Ok(None),
            e => Err(e),
        }
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

    /// Publish the given robot's position.
    ///
    /// `x`/`y`/`z` are meters in the site frame named by `frame` (use
    /// `"map"` for the default site frame, or `"wgs84"` for GPS
    /// coordinates: `x` = longitude, `y` = latitude, `z` = altitude in
    /// meters). `heading` is radians counter-clockwise from east (the
    /// +x axis). The payload is a JSON object
    /// `{"ts_us", "frame", "x", "y", "z", "heading"?}` stamped with the
    /// fabric clock.
    ///
    /// Feed this from your localization loop (odometry, SLAM, GPS) — it
    /// is safe to call at any rate; publishes are throttled to 5 Hz and
    /// excess calls are dropped (returning `Ok`).
    pub fn set_pose(
        &self,
        name: &str,
        x: f64,
        y: f64,
        z: f64,
        frame: &str,
        heading: Option<f64>,
    ) -> Result<()> {
        if !(x.is_finite() && y.is_finite() && z.is_finite())
            || heading.is_some_and(|h| !h.is_finite())
        {
            return Err(Error::Invalid(
                "set_pose: coordinates must be finite".into(),
            ));
        }
        {
            let mut last = self.pose_last_pub.lock().unwrap();
            let now = Instant::now();
            if last.is_some_and(|t| now.duration_since(t) < POSE_MIN_INTERVAL) {
                return Ok(());
            }
            *last = Some(now);
        }
        let heading_field = heading
            .map(|h| format!(",\"heading\":{h}"))
            .unwrap_or_default();
        let payload = format!(
            "{{\"ts_us\":{},\"frame\":{},\"x\":{x},\"y\":{y},\"z\":{z}{heading_field}}}",
            crate::fabric_now_us(),
            json_quote(frame),
        );
        self.put(
            &format!("{name}/pose"),
            payload.as_bytes(),
            PublishOptions {
                priority: Priority::DATA,
                express: false,
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

    /// Declare a callback-based subscriber on `key`.
    ///
    /// The callback runs on the SDK receive thread. Keep it short or hand
    /// work off to another thread/channel.
    pub fn subscribe_with<F>(&self, key: &str, callback: F) -> Result<CallbackSubscriber<'_>>
    where
        F: Fn(Sample) + Send + Sync + 'static,
    {
        let key = CString::new(key)?;
        let mut state = Box::new(CallbackState {
            callback: Box::new(callback),
        });
        let raw = unsafe {
            adamo_sys::adamo_subscribe_cb(
                self.raw.as_ptr(),
                key.as_ptr(),
                Some(callback_trampoline),
                state.as_mut() as *mut CallbackState as *mut c_void,
            )
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(CallbackSubscriber {
                raw,
                _state: state,
                _session: PhantomData,
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// One-shot query. Collects all replies arriving within `timeout`.
    pub fn get(&self, key: &str, timeout: Duration) -> Result<Vec<Sample>> {
        let key = CString::new(key)?;
        let mut count = 0usize;
        let raw = unsafe {
            adamo_sys::adamo_get(
                self.raw.as_ptr(),
                key.as_ptr(),
                timeout.as_millis().min(u64::MAX as u128) as u64,
                &mut count,
            )
        };
        if raw.is_null() {
            return match last_ffi_error() {
                Error::Ffi(m) if m == "(no error message)" && count == 0 => Ok(Vec::new()),
                e => Err(e),
            };
        }

        let raw_samples = unsafe { slice::from_raw_parts(raw, count) };
        let mut samples = Vec::with_capacity(count);
        for sample in raw_samples {
            if let Some(sample) = Sample::from_borrowed(*sample) {
                samples.push(sample);
            }
        }
        unsafe { adamo_sys::adamo_get_replies_free(raw, count) };
        Ok(samples)
    }

    /// Declare this client alive at `{token_key}/alive`.
    pub fn alive(&self, token_key: &str) -> Result<LivelinessToken<'_>> {
        let token_key = CString::new(token_key)?;
        let raw = unsafe {
            adamo_sys::adamo_liveliness_declare(self.raw.as_ptr(), token_key.as_ptr())
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(LivelinessToken {
                raw,
                _session: PhantomData,
            }),
            None => Err(last_ffi_error()),
        }
    }

    /// Query currently-live tokens matching `pattern`.
    pub fn live_tokens(&self, pattern: &str) -> Result<Vec<String>> {
        let pattern = CString::new(pattern)?;
        let mut count = 0usize;
        let raw = unsafe {
            adamo_sys::adamo_liveliness_get(self.raw.as_ptr(), pattern.as_ptr(), &mut count)
        };
        if raw.is_null() {
            return match last_ffi_error() {
                Error::Ffi(m) if m == "(no error message)" && count == 0 => Ok(Vec::new()),
                e => Err(e),
            };
        }

        let raw_tokens = unsafe { slice::from_raw_parts(raw, count) };
        let mut tokens = Vec::with_capacity(count);
        let mut invalid_utf8 = false;
        for token in raw_tokens {
            let token = unsafe { CStr::from_ptr(*token) };
            match token.to_str() {
                Ok(token) => tokens.push(token.to_owned()),
                Err(_) => invalid_utf8 = true,
            }
        }
        unsafe { adamo_sys::adamo_liveliness_tokens_free(raw, count) };
        if invalid_utf8 {
            return Err(Error::InvalidUtf8);
        }
        Ok(tokens)
    }

    /// Watch for liveliness changes.
    ///
    /// If `history` is true, the current set of live tokens is delivered
    /// up front before subsequent changes.
    pub fn on_liveliness<F>(
        &self,
        pattern: &str,
        history: bool,
        callback: F,
    ) -> Result<LivelinessSubscriber<'_>>
    where
        F: Fn(String, bool) + Send + Sync + 'static,
    {
        let pattern = CString::new(pattern)?;
        let mut state = Box::new(LivelinessState {
            callback: Box::new(callback),
        });
        let raw = unsafe {
            adamo_sys::adamo_liveliness_subscribe(
                self.raw.as_ptr(),
                pattern.as_ptr(),
                history as i32,
                Some(liveliness_trampoline),
                state.as_mut() as *mut LivelinessState as *mut c_void,
            )
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(LivelinessSubscriber {
                raw,
                _state: state,
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

/// Priority constants shared with the Python SDK.
///
/// Values are on the native 0-255 scale; higher is more important.
pub struct Priority;

impl Priority {
    pub const REAL_TIME: u8 = 250;
    pub const INTERACTIVE_HIGH: u8 = 220;
    pub const INTERACTIVE_LOW: u8 = 190;
    pub const DATA_HIGH: u8 = 150;
    pub const DATA: u8 = 100;
    pub const DATA_LOW: u8 = 80;
    pub const BACKGROUND: u8 = 20;
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            priority: Priority::DATA,
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
            priority: Priority::DATA,
            express: false,
            reliable: false,
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

type SampleCallback = dyn Fn(Sample) + Send + Sync + 'static;

struct CallbackState {
    callback: Box<SampleCallback>,
}

unsafe extern "C" fn callback_trampoline(
    sample: *const adamo_sys::adamo_sample_t,
    user: *mut c_void,
) {
    if sample.is_null() || user.is_null() {
        return;
    }
    let Some(sample) = Sample::from_borrowed(sample) else {
        return;
    };
    let state = unsafe { &*(user as *const CallbackState) };
    if catch_unwind(AssertUnwindSafe(|| (state.callback)(sample))).is_err() {
        eprintln!("adamo subscribe_with callback panicked");
    }
}

/// A callback-based subscriber. Tied to its parent [`Session`] by lifetime.
pub struct CallbackSubscriber<'a> {
    raw: NonNull<adamo_sys::adamo_cb_sub_t>,
    _state: Box<CallbackState>,
    _session: PhantomData<&'a Session>,
}

unsafe impl Send for CallbackSubscriber<'_> {}
unsafe impl Sync for CallbackSubscriber<'_> {}

impl Drop for CallbackSubscriber<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_cb_sub_free(self.raw.as_ptr()) };
    }
}

/// A liveliness token. Drop it to undeclare the token.
pub struct LivelinessToken<'a> {
    raw: NonNull<adamo_sys::adamo_liveliness_token_t>,
    _session: PhantomData<&'a Session>,
}

unsafe impl Send for LivelinessToken<'_> {}
unsafe impl Sync for LivelinessToken<'_> {}

impl Drop for LivelinessToken<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_liveliness_token_free(self.raw.as_ptr()) };
    }
}

type LivelinessCallback = dyn Fn(String, bool) + Send + Sync + 'static;

struct LivelinessState {
    callback: Box<LivelinessCallback>,
}

unsafe extern "C" fn liveliness_trampoline(
    key: *const c_char,
    alive: i32,
    user: *mut c_void,
) {
    if key.is_null() || user.is_null() {
        return;
    }
    let key = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let state = unsafe { &*(user as *const LivelinessState) };
    if catch_unwind(AssertUnwindSafe(|| (state.callback)(key, alive != 0))).is_err() {
        eprintln!("adamo on_liveliness callback panicked");
    }
}

/// A liveliness watcher. Tied to its parent [`Session`] by lifetime.
pub struct LivelinessSubscriber<'a> {
    raw: NonNull<adamo_sys::adamo_liveliness_sub_t>,
    _state: Box<LivelinessState>,
    _session: PhantomData<&'a Session>,
}

unsafe impl Send for LivelinessSubscriber<'_> {}
unsafe impl Sync for LivelinessSubscriber<'_> {}

impl Drop for LivelinessSubscriber<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_liveliness_sub_free(self.raw.as_ptr()) };
    }
}

/// An owned, decoded sample.
#[derive(Debug, Clone)]
pub struct Sample {
    pub key: String,
    pub payload: Vec<u8>,
    pub is_delete: bool,
    pub timestamp_us: Option<u64>,
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
            let timestamp_us = (s.timestamp_us != 0).then_some(s.timestamp_us);
            adamo_sys::adamo_sample_free(raw);
            Sample {
                key,
                payload,
                is_delete,
                timestamp_us,
            }
        };
        Some(sample)
    }

    /// Copy a borrowed `adamo_sample_t` supplied by a callback.
    fn from_borrowed(raw: *const adamo_sys::adamo_sample_t) -> Option<Self> {
        if raw.is_null() {
            return None;
        }
        let s = unsafe { &*raw };
        let key = if s.key.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(s.key).to_string_lossy().into_owned() }
        };
        let payload = if s.payload.is_null() || s.payload_len == 0 {
            Vec::new()
        } else {
            unsafe { slice::from_raw_parts(s.payload, s.payload_len).to_vec() }
        };
        Some(Sample {
            key,
            payload,
            is_delete: s.is_delete != 0,
            timestamp_us: (s.timestamp_us != 0).then_some(s.timestamp_us),
        })
    }
}
