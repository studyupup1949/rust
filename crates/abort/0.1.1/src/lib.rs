#![no_std]

extern "C" {
    #[link_name = "abort"]
    fn c_abort() -> !;
}

#[inline(always)]
pub fn abort() -> ! { unsafe { c_abort() } }
