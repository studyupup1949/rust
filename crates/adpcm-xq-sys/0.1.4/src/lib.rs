use std::ffi::{c_int, c_void};

// Bindings to the adpcm-xq encoder
unsafe extern "C" {
    pub fn adpcm_create_context(num_channels: c_int, lookahead: c_int, noise_shaping: c_int, initial_deltas: *mut i32) -> *mut c_void;
    pub fn adpcm_encode_block(p: *mut c_void, outbuf: *mut u8, outbufsize: *mut usize, inbuf: *const i16, inbufcount: c_int) -> c_int;
    pub fn adpcm_decode_block(outbuf: *mut i16, inbuf: *const u8, inbufsize: usize, channels: c_int) -> c_int;
    pub fn adpcm_free_context(p: *mut c_void);
}