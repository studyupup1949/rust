use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::error::{Error, Result, last_ffi_error};
use crate::session::Protocol;

/// A robot builder — declare video tracks, then call [`Robot::run`] from
/// a dedicated thread to drive the encoding + transport pipeline.
pub struct Robot {
    raw: NonNull<adamo_sys::adamo_robot_t>,
}

unsafe impl Send for Robot {}

impl Robot {
    pub fn new(api_key: &str, name: Option<&str>, protocol: Protocol) -> Result<Self> {
        let api_key = CString::new(api_key)?;
        let name_c = match name {
            Some(n) => Some(CString::new(n)?),
            None => None,
        };
        let name_ptr = name_c
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        let raw = unsafe {
            adamo_sys::adamo_robot_new(api_key.as_ptr(), name_ptr, protocol_raw(protocol))
        };
        NonNull::new(raw)
            .map(|raw| Robot { raw })
            .ok_or_else(last_ffi_error)
    }

    /// Attach a caller-fed video track. Push raw frames via
    /// [`VideoTrack::send`].
    pub fn video(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixel_format: &str,
        fps: u32,
        bitrate_kbps: u32,
    ) -> Result<VideoTrack<'_>> {
        let name = CString::new(name)?;
        let fmt = CString::new(pixel_format)?;
        let raw = unsafe {
            adamo_sys::adamo_robot_video(
                self.raw.as_ptr(),
                name.as_ptr(),
                width,
                height,
                fmt.as_ptr(),
                fps,
                bitrate_kbps,
            )
        };
        NonNull::new(raw)
            .map(|raw| VideoTrack {
                raw,
                _robot: PhantomData,
            })
            .ok_or_else(last_ffi_error)
    }

    /// Attach a V4L2 video source (Linux only). The Rust side owns the
    /// capture thread; no frames cross back into Rust.
    pub fn attach_v4l2(
        &mut self,
        name: &str,
        device: &str,
        width: u32,
        height: u32,
        fps: u32,
        bitrate_kbps: u32,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let device = CString::new(device)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_v4l2(
                self.raw.as_ptr(),
                name.as_ptr(),
                device.as_ptr(),
                width,
                height,
                fps,
                bitrate_kbps,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach a GStreamer pipeline source. The pipeline must produce raw
    /// video matching the encoder input caps.
    pub fn attach_gst(
        &mut self,
        name: &str,
        pipeline: &str,
        width: u32,
        height: u32,
        fps: u32,
        bitrate_kbps: u32,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let pipeline = CString::new(pipeline)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_gst(
                self.raw.as_ptr(),
                name.as_ptr(),
                pipeline.as_ptr(),
                width,
                height,
                fps,
                bitrate_kbps,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Consume the robot and block driving the pipeline. Returns on
    /// clean shutdown (currently the pipeline never self-exits).
    pub fn run(self) -> Result<()> {
        // Take the raw pointer out of self so Drop doesn't free it
        // behind `run`'s back.
        let raw = self.raw.as_ptr();
        std::mem::forget(self);
        let rc = unsafe { adamo_sys::adamo_robot_run(raw) };
        // adamo_robot_run consumes the robot on the C side — don't free.
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }
}

impl Drop for Robot {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_robot_free(self.raw.as_ptr()) };
    }
}

/// A video track registered on a [`Robot`]. Raw frames are pushed via
/// [`VideoTrack::send`].
pub struct VideoTrack<'a> {
    raw: NonNull<adamo_sys::adamo_video_track_t>,
    _robot: PhantomData<&'a mut Robot>,
}

unsafe impl Send for VideoTrack<'_> {}

impl VideoTrack<'_> {
    pub fn send(&mut self, frame: &[u8]) -> Result<()> {
        let rc = unsafe {
            adamo_sys::adamo_video_track_send(self.raw.as_ptr(), frame.as_ptr(), frame.len())
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }
}

impl Drop for VideoTrack<'_> {
    fn drop(&mut self) {
        unsafe { adamo_sys::adamo_video_track_free(self.raw.as_ptr()) };
    }
}

/// Best H.264 encoder element factory available on this host
/// (`"nvh264enc"`, `"vtenc_h264"`, …) or `"none"`.
pub fn detect_encoder() -> Result<&'static str> {
    let ptr = unsafe { adamo_sys::adamo_detect_encoder() };
    if ptr.is_null() {
        return Err(last_ffi_error());
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| Error::InvalidUtf8)
}

fn protocol_raw(p: Protocol) -> adamo_sys::adamo_protocol_t {
    match p {
        Protocol::Udp => adamo_sys::ADAMO_PROTOCOL_UDP,
        Protocol::Quic => adamo_sys::ADAMO_PROTOCOL_QUIC,
        Protocol::Tcp => adamo_sys::ADAMO_PROTOCOL_TCP,
    }
}
