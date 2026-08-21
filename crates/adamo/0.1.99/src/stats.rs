//! Latency / network-stats helpers.
//!
//! Robots running the Adamo binary publish a 1 Hz heartbeat carrying a
//! GARCH-based forecast of the current network regime, and echo a
//! ping/pong topic for round-trip-time probing. This module exposes both
//! as typed Rust APIs so callers don't have to know the heartbeat JSON
//! shape.

use std::ffi::CString;
use std::time::Duration;

use crate::error::{Error, Result, last_ffi_error};
use crate::session::{Session, Subscriber};

/// Network regime as classified by the robot's GARCH forecaster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Stable,
    Degrading,
    Volatile,
    Recovering,
}

impl Regime {
    fn from_raw(v: adamo_sys::adamo_regime_t) -> Self {
        match v {
            adamo_sys::ADAMO_REGIME_DEGRADING => Self::Degrading,
            adamo_sys::ADAMO_REGIME_VOLATILE => Self::Volatile,
            adamo_sys::ADAMO_REGIME_RECOVERING => Self::Recovering,
            _ => Self::Stable,
        }
    }
}

/// One latency snapshot from a robot's heartbeat.
#[derive(Debug, Clone, Copy)]
pub struct LatencyStats {
    pub regime: Regime,
    pub jitter_hint_ms: f32,
    pub garch_sigma_ms: f32,
    pub target_bitrate_kbps: u32,
    pub loss_rate: f32,
    pub queuing_delay_ms: f32,
    pub timestamp_ms: u64,
}

impl LatencyStats {
    /// Parse a heartbeat payload (received on the topic returned by
    /// [`heartbeat_topic`]). Returns `None` if the payload isn't valid
    /// JSON or has no `forecast` block — callers should treat that as
    /// "no stats yet" rather than an error.
    pub fn parse(payload: &[u8]) -> Option<Self> {
        let mut out = adamo_sys::adamo_latency_stats_t {
            regime: 0,
            jitter_hint_ms: 0.0,
            garch_sigma_ms: 0.0,
            target_bitrate_kbps: 0,
            loss_rate: 0.0,
            queuing_delay_ms: 0.0,
            timestamp_ms: 0,
        };
        let rc = unsafe {
            adamo_sys::adamo_parse_heartbeat(payload.as_ptr(), payload.len(), &mut out)
        };
        if rc != 0 {
            return None;
        }
        Some(Self {
            regime: Regime::from_raw(out.regime),
            jitter_hint_ms: out.jitter_hint_ms,
            garch_sigma_ms: out.garch_sigma_ms,
            target_bitrate_kbps: out.target_bitrate_kbps,
            loss_rate: out.loss_rate,
            queuing_delay_ms: out.queuing_delay_ms,
            timestamp_ms: out.timestamp_ms,
        })
    }
}

fn topic_via<F>(robot: &str, raw: F) -> Result<String>
where
    F: Fn(*const std::os::raw::c_char, *mut std::os::raw::c_char, usize) -> isize,
{
    let robot_c = CString::new(robot)?;
    let needed = raw(robot_c.as_ptr(), std::ptr::null_mut(), 0);
    if needed < 0 {
        return Err(last_ffi_error());
    }
    let mut buf = vec![0u8; needed as usize + 1];
    let written = raw(
        robot_c.as_ptr(),
        buf.as_mut_ptr() as *mut std::os::raw::c_char,
        buf.len(),
    );
    if written < 0 {
        return Err(last_ffi_error());
    }
    buf.truncate(written as usize);
    String::from_utf8(buf).map_err(|_| Error::InvalidUtf8)
}

/// The heartbeat topic carrying [`LatencyStats`] for `robot`.
pub fn heartbeat_topic(robot: &str) -> Result<String> {
    topic_via(robot, |r, o, c| unsafe { adamo_sys::adamo_heartbeat_topic(r, o, c) })
}

/// The ping topic for `robot`. Used by [`Session::measure_rtt`].
pub fn ping_topic(robot: &str) -> Result<String> {
    topic_via(robot, |r, o, c| unsafe { adamo_sys::adamo_ping_topic(r, o, c) })
}

/// The pong topic the robot echoes back on.
pub fn pong_topic(robot: &str) -> Result<String> {
    topic_via(robot, |r, o, c| unsafe { adamo_sys::adamo_pong_topic(r, o, c) })
}

impl Session {
    /// Measure round-trip time to `robot` by publishing a ping and
    /// waiting for the echoed pong. Blocks the calling thread.
    pub fn measure_rtt(&self, robot: &str, timeout: Duration) -> Result<Duration> {
        let robot_c = CString::new(robot)?;
        let mut out_us: u64 = 0;
        let timeout_ms = timeout.as_millis().min(u64::MAX as u128) as u64;
        let rc = unsafe {
            adamo_sys::adamo_measure_rtt(
                self.raw_ptr(),
                robot_c.as_ptr(),
                timeout_ms,
                &mut out_us,
            )
        };
        if rc == 0 {
            Ok(Duration::from_micros(out_us))
        } else {
            Err(last_ffi_error())
        }
    }

    /// Subscribe to the heartbeat topic for `robot`. Each
    /// [`Subscriber::recv`] returns a raw heartbeat sample whose
    /// payload can be parsed with [`LatencyStats::parse`]. Returns
    /// `None` on the very first heartbeat (before the forecast loop has
    /// produced its first sample).
    pub fn watch_latency(&self, robot: &str) -> Result<Subscriber<'_>> {
        self.subscribe(&heartbeat_topic(robot)?)
    }
}
