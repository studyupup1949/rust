use crate::preclude::AbiString;
use std::alloc::{Layout, alloc, dealloc};
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "C" fn __alloc(size: usize) -> *mut c_void {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { alloc(layout) as *mut c_void }
}

#[unsafe(no_mangle)]
pub extern "C" fn __dealloc(ptr: *mut c_void, size: usize) {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { dealloc(ptr as *mut u8, layout) }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C" fn __free_AbiString(ptr: AbiString) -> bool {
    unsafe { ptr.drop().is_ok() }
}
