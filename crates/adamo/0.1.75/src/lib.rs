//! Rust SDK for the Adamo Network.
//!
//! This crate is a safe, idiomatic wrapper over the precompiled Adamo
//! shared library (shipped via [`adamo-sys`]).
//!
//! ```no_run
//! use adamo::Session;
//!
//! let session = Session::open_default("my-api-key")?;
//! session.put("telemetry/heartbeat", b"hello", Default::default())?;
//! # Ok::<_, adamo::Error>(())
//! ```

mod error;
pub mod control;
mod session;
pub mod stats;

#[cfg(feature = "video")]
pub mod camera;

#[cfg(feature = "video")]
mod robot;

pub use error::{Error, Result};
pub use control::{ControlMessage, JointState, Joy, JoystickCommand, decode_control};
pub use session::{
    CallbackSubscriber, LivelinessSubscriber, LivelinessToken, Priority, Protocol, PublishOptions,
    Publisher, PublisherOptions, Sample, Session, Subscriber,
};
pub use stats::{LatencyStats, Regime, heartbeat_topic, ping_topic, pong_topic};

/// Microseconds since the Unix epoch on the **adamo fabric clock** — the
/// shared time axis every robot, browser, and relay on this network sees.
///
/// Backed by `zenoh-plugin-adamo-time` running on each router and a
/// background ping/pong that maintains the offset estimate. Falls back to
/// the local wall clock until the first sync completes (usually <100 ms
/// after [`Session::open`]). Check [`fabric_synced`] to distinguish.
///
/// Use this instead of `SystemTime::now()` whenever a timestamp will be
/// subtracted from a stamp produced on a different node.
pub fn fabric_now_us() -> u64 {
    // SAFETY: free function, no preconditions.
    unsafe { adamo_sys::adamo_fabric_now_us() }
}

/// `true` once the client has completed at least one ping/pong with the
/// timehub plugin and [`fabric_now_us`] is reading from the synced clock
/// rather than the local wall-clock fallback.
pub fn fabric_synced() -> bool {
    unsafe { adamo_sys::adamo_fabric_synced() != 0 }
}

#[cfg(feature = "video")]
pub use robot::{Robot, VideoBackend, VideoOptions, VideoTrack, detect_encoder};

#[cfg(feature = "video")]
pub use camera::{AttachOptions, CameraInfo, attach_all, attach_camera, discover, discover_v4l2};
