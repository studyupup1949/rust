use std::ffi::{c_double, c_int, c_void};

// Noise shaping
/// flat noise (no shaping)
pub const NOISE_SHAPING_OFF: c_int = 0;
/// static 1st-order shaping (configurable, highpass default)
pub const NOISE_SHAPING_STATIC: c_int = 0x100;
/// dynamically tilted noise based on signal
pub const NOISE_SHAPING_DYNAMIC: c_int = 0x200;

// Lookahead
/// depth of search
pub const LOOKAHEAD_DEPTH: c_int = 0x0ff;
/// full breadth of search (all branches taken)
pub const LOOKAHEAD_EXHAUSTIVE: c_int = 0x800;
/// no branches taken (internal use only!)
pub const LOOKAHEAD_NO_BRANCHING: c_int = 0x400;

// Bindings to the adpcm-xq encoder
unsafe extern "C" {
    /* adpcm-lib.c */

    pub fn adpcm_sample_count_to_block_size(sample_count: c_int, num_chans: c_int, bps: c_int) -> c_int;
    pub fn adpcm_block_size_to_sample_count(block_size: c_int, num_chans: c_int, bps: c_int) -> c_int;
    pub fn adpcm_align_block_size(block_size: c_int, num_chans: c_int, bps: c_int, round_up: c_int) -> c_int;
    pub fn adpcm_create_context(num_channels: c_int, sample_rate: c_int, lookahead: c_int, noise_shaping: c_int) -> *mut c_void;
    pub fn adpcm_set_shaping_weight(p: *mut c_void, shaping_weight: c_double);
    pub fn adpcm_encode_block_ex(p: *mut c_void, outbuf: *mut u8, outbufsize: *mut usize, inbuf: *const i16, inbufcount: c_int, bps: c_int) -> c_int;
    pub fn adpcm_encode_block(p: *mut c_void, outbuf: *mut u8, outbufsize: *mut usize, inbuf: *const i16, inbufcount: c_int) -> c_int;
    pub fn adpcm_decode_block_ex(outbuf: *mut i16, inbuf: *const u8, inbufsize: usize, channels: c_int, bps: c_int) -> c_int;
    pub fn adpcm_decode_block(outbuf: *mut i16, inbuf: *const u8, inbufsize: usize, channels: c_int) -> c_int;
    pub fn adpcm_free_context(p: *mut c_void);

    /* adpcm-dns.c */

    pub fn generate_dns_values(samples: *const i16, sample_count: c_int, num_chans: c_int, sample_rate: c_int,
        values: *mut i16, min_value: i16, last_value: i16);
}