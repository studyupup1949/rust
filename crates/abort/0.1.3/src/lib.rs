#![no_std]

#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

//! Abnormal termination: see [`abort`].

/// Abnormally terminate the process.
#[inline(always)]
pub fn abort() -> ! { a() }

#[cfg(feature = "nightly")]
#[inline(always)]
fn a() -> ! { core::intrinsics::abort() }

#[cfg(not(feature = "nightly"))]
#[inline(always)]
fn a() -> ! {
    struct A;

    impl Drop for A {
        #[inline(always)]
        fn drop(&mut self) { panic!() }
    }

    let _a = A;
    panic!()
}
