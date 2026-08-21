use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::error::{Error, Result, last_ffi_error};
use crate::session::Protocol;

/// Video backend selection for SDK-managed video tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoBackend {
    /// Use the SDK default for the source type.
    #[default]
    Auto,
    /// Force the GStreamer source/encoder backend.
    GStreamer,
    /// Force the native hardware pipeline backend.
    HwPipeline,
}

/// Full video option set shared with the Python SDK.
///
/// `pixel_format` identifies the input carried by caller-fed and shared-memory
/// tracks. Use a raw layout such as `"BGRA"` or `"NV12"`, or `"h264"` / `"h265"`
/// for encoded access units that should be transcoded. It is optional for V4L2
/// and GStreamer sources, where it acts as a source-format hint.
#[derive(Debug, Clone)]
pub struct VideoOptions {
    pub width: u32,
    pub height: u32,
    pub pixel_format: Option<String>,
    pub codec: String,
    pub encoder: Option<String>,
    pub bitrate_kbps: u32,
    pub adaptive_bitrate: bool,
    pub min_bitrate_kbps: Option<u32>,
    pub max_bitrate_kbps: Option<u32>,
    pub bitrate_priority: f32,
    pub fps: u32,
    pub keyframe_distance: f64,
    pub stereo: bool,
    pub backend: VideoBackend,
    /// Optional iceoryx2 service name to also publish every raw captured
    /// frame to. Applies to `attach_v4l2_with_options` tracks only; other
    /// source types log a robot-side warning and run without the tee.
    /// Frames are headerless raw bytes in the negotiated capture format.
    pub shm_publish: Option<String>,
    /// Forward an already-encoded H.264/H.265 bitstream without re-encoding.
    /// The source must already emit the chosen `codec`; set `codec`/`encoder`
    /// to tag H.265. No encoder runs in this mode.
    pub passthrough: bool,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            pixel_format: None,
            codec: "h264".to_string(),
            encoder: None,
            bitrate_kbps: 2000,
            adaptive_bitrate: true,
            min_bitrate_kbps: None,
            max_bitrate_kbps: None,
            bitrate_priority: 1.0,
            fps: 30,
            keyframe_distance: 2.0,
            stereo: false,
            backend: VideoBackend::Auto,
            shm_publish: None,
            passthrough: false,
        }
    }
}

impl VideoOptions {
    pub fn with_pixel_format(mut self, pixel_format: impl Into<String>) -> Self {
        self.pixel_format = Some(pixel_format.into());
        self
    }

    /// Set the source format. This is an alias for [`Self::with_pixel_format`]
    /// whose name also covers encoded shared-memory inputs such as H.264/H.265.
    pub fn with_source_format(self, source_format: impl Into<String>) -> Self {
        self.with_pixel_format(source_format)
    }

    pub fn with_encoder(mut self, encoder: impl Into<String>) -> Self {
        self.encoder = Some(encoder.into());
        self
    }

    pub fn with_backend(mut self, backend: VideoBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_shm_publish(mut self, service: impl Into<String>) -> Self {
        self.shm_publish = Some(service.into());
        self
    }

    pub fn with_passthrough(mut self, passthrough: bool) -> Self {
        self.passthrough = passthrough;
        self
    }
}

struct RawVideoOptions {
    raw: adamo_sys::adamo_video_options_t,
    _pixel_format: Option<CString>,
    _codec: CString,
    _encoder: Option<CString>,
    _shm_publish: Option<CString>,
}

impl RawVideoOptions {
    fn new(options: &VideoOptions) -> Result<Self> {
        let pixel_format = options
            .pixel_format
            .as_deref()
            .map(CString::new)
            .transpose()?;
        let codec = CString::new(options.codec.as_str())?;
        let encoder = options.encoder.as_deref().map(CString::new).transpose()?;
        let shm_publish = options
            .shm_publish
            .as_deref()
            .map(CString::new)
            .transpose()?;

        let mut raw = unsafe { adamo_sys::adamo_video_options_default() };
        raw.width = options.width;
        raw.height = options.height;
        raw.pixel_format = pixel_format
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        raw.codec = codec.as_ptr();
        raw.encoder = encoder
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        raw.bitrate_kbps = options.bitrate_kbps;
        raw.adaptive_bitrate = if options.adaptive_bitrate { 1 } else { 0 };
        raw.min_bitrate_kbps = options.min_bitrate_kbps.unwrap_or(0);
        raw.max_bitrate_kbps = options.max_bitrate_kbps.unwrap_or(0);
        raw.bitrate_priority = options.bitrate_priority;
        raw.fps = options.fps;
        raw.keyframe_distance = options.keyframe_distance;
        raw.stereo = if options.stereo { 1 } else { 0 };
        raw.backend = backend_raw(options.backend);
        // Set unconditionally: an older prebuilt libadamo returns a struct
        // without this field from adamo_video_options_default(), leaving the
        // appended slot uninitialized otherwise.
        raw.shm_publish = shm_publish
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        // Set unconditionally for the same reason as shm_publish: an older
        // prebuilt libadamo leaves this appended slot uninitialized.
        raw.passthrough = if options.passthrough { 1 } else { 0 };

        Ok(Self {
            raw,
            _pixel_format: pixel_format,
            _codec: codec,
            _encoder: encoder,
            _shm_publish: shm_publish,
        })
    }

    fn as_ptr(&self) -> *const adamo_sys::adamo_video_options_t {
        &self.raw
    }
}

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

    pub fn new_default(api_key: &str, name: Option<&str>) -> Result<Self> {
        let api_key = CString::new(api_key)?;
        let name_c = match name {
            Some(n) => Some(CString::new(n)?),
            None => None,
        };
        let name_ptr = name_c
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        let raw = unsafe { adamo_sys::adamo_robot_new_default(api_key.as_ptr(), name_ptr) };
        NonNull::new(raw)
            .map(|raw| Robot { raw })
            .ok_or_else(last_ffi_error)
    }

    /// Attach a caller-fed video track. Push raw frames via
    /// [`VideoTrack::send`].
    ///
    /// Set `stereo = true` for side-by-side stereo input (e.g. ZED), so
    /// downstream consumers split the frame into left/right eyes.
    pub fn video(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixel_format: &str,
        fps: u32,
        bitrate_kbps: u32,
        stereo: bool,
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
                stereo,
            )
        };
        NonNull::new(raw)
            .map(|raw| VideoTrack {
                raw,
                _robot: PhantomData,
            })
            .ok_or_else(last_ffi_error)
    }

    /// Attach a caller-fed video track with the full Python-compatible option
    /// set.
    pub fn video_with_options(
        &mut self,
        name: &str,
        options: &VideoOptions,
    ) -> Result<VideoTrack<'_>> {
        let name = CString::new(name)?;
        let options = RawVideoOptions::new(options)?;
        let raw = unsafe {
            adamo_sys::adamo_robot_video_configured(
                self.raw.as_ptr(),
                name.as_ptr(),
                options.as_ptr(),
            )
        };
        NonNull::new(raw)
            .map(|raw| VideoTrack {
                raw,
                _robot: PhantomData,
            })
            .ok_or_else(last_ffi_error)
    }

    /// Attach a caller-fed video track with explicit encoder/backend choices.
    ///
    /// Set `hw_pipeline = false` to route frames through the GStreamer
    /// pipeline, for example Linux VA-API with `encoder = "vah264enc"`.
    pub fn video_with_encoder(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        pixel_format: &str,
        fps: u32,
        bitrate_kbps: u32,
        stereo: bool,
        encoder: &str,
        hw_pipeline: bool,
    ) -> Result<VideoTrack<'_>> {
        let name = CString::new(name)?;
        let fmt = CString::new(pixel_format)?;
        let encoder = CString::new(encoder)?;
        let raw = unsafe {
            adamo_sys::adamo_robot_video_with_options(
                self.raw.as_ptr(),
                name.as_ptr(),
                width,
                height,
                fmt.as_ptr(),
                fps,
                bitrate_kbps,
                stereo,
                encoder.as_ptr(),
                hw_pipeline,
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
    ///
    /// Set `stereo = true` for side-by-side stereo cameras (e.g. ZED).
    pub fn attach_v4l2(
        &mut self,
        name: &str,
        device: &str,
        width: u32,
        height: u32,
        fps: u32,
        bitrate_kbps: u32,
        stereo: bool,
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
                stereo,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach a V4L2 video source with the full Python-compatible option set.
    pub fn attach_v4l2_with_options(
        &mut self,
        name: &str,
        device: &str,
        options: &VideoOptions,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let device = CString::new(device)?;
        let options = RawVideoOptions::new(options)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_v4l2_configured(
                self.raw.as_ptr(),
                name.as_ptr(),
                device.as_ptr(),
                options.as_ptr(),
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach a GStreamer pipeline source. The pipeline must produce raw
    /// video matching the encoder input caps.
    ///
    /// Set `stereo = true` if the pipeline yields side-by-side stereo frames.
    pub fn attach_gst(
        &mut self,
        name: &str,
        pipeline: &str,
        width: u32,
        height: u32,
        fps: u32,
        bitrate_kbps: u32,
        stereo: bool,
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
                stereo,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach a GStreamer pipeline source with the full Python-compatible
    /// option set.
    pub fn attach_gst_with_options(
        &mut self,
        name: &str,
        pipeline: &str,
        options: &VideoOptions,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let pipeline = CString::new(pipeline)?;
        let options = RawVideoOptions::new(options)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_gst_configured(
                self.raw.as_ptr(),
                name.as_ptr(),
                pipeline.as_ptr(),
                options.as_ptr(),
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach an existing iceoryx2 shared-memory video source. The source must
    /// publish one complete raw frame or encoded access unit per `[u8]` sample,
    /// with no Adamo-specific headers or metadata prepended.
    ///
    /// `pixel_format` must match the producer's payload, for example `"BGRA"`,
    /// `"NV12"`, `"mjpeg"`, `"h264"`, or `"h265"`. Encoded input is decoded
    /// and re-encoded unless [`VideoOptions::passthrough`] is enabled through
    /// [`Self::attach_shm_with_options`].
    pub fn attach_shm(
        &mut self,
        name: &str,
        service: &str,
        width: u32,
        height: u32,
        pixel_format: &str,
        fps: u32,
        bitrate_kbps: u32,
        stereo: bool,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let service = CString::new(service)?;
        let pixel_format = CString::new(pixel_format)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_shm(
                self.raw.as_ptr(),
                name.as_ptr(),
                service.as_ptr(),
                width,
                height,
                pixel_format.as_ptr(),
                fps,
                bitrate_kbps,
                stereo,
            )
        };
        if rc == 0 { Ok(()) } else { Err(last_ffi_error()) }
    }

    /// Attach an existing iceoryx2 shared-memory video source with the full
    /// option set. Set `options.pixel_format` to `"h264"` or `"h265"` and leave
    /// `options.passthrough` false to transcode encoded access units.
    pub fn attach_shm_with_options(
        &mut self,
        name: &str,
        service: &str,
        options: &VideoOptions,
    ) -> Result<()> {
        let name = CString::new(name)?;
        let service = CString::new(service)?;
        let options = RawVideoOptions::new(options)?;
        let rc = unsafe {
            adamo_sys::adamo_robot_attach_video_shm_configured(
                self.raw.as_ptr(),
                name.as_ptr(),
                service.as_ptr(),
                options.as_ptr(),
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

fn backend_raw(backend: VideoBackend) -> adamo_sys::adamo_video_backend_t {
    match backend {
        VideoBackend::Auto => adamo_sys::ADAMO_VIDEO_BACKEND_AUTO,
        VideoBackend::GStreamer => adamo_sys::ADAMO_VIDEO_BACKEND_GSTREAMER,
        VideoBackend::HwPipeline => adamo_sys::ADAMO_VIDEO_BACKEND_HW_PIPELINE,
    }
}
