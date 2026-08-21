#![feature(fn_traits)]

use std::env::Args;
pub struct RussTestDescAndFn {
    pub desc: rustc_test::TestDesc,
    pub testfn: fn(),
}
