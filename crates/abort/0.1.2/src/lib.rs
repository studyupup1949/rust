#![no_std]

//! Abnormal termination: see [`abort`].

struct A;

impl Drop for A {
    #[inline(always)]
    fn drop(&mut self) { panic!() }
}

/// Abnormally terminate the process.
#[inline(always)]
pub fn abort() -> ! {
    let _a = A;
    panic!()
}
