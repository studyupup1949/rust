use crate::error::print_error;
use crate::preclude::{AbiString, Error};
use std::alloc::{Layout, alloc, dealloc};
use std::any::TypeId;

struct AllocBytes {
    ptr: *mut u8,
    size: usize,
}
impl AllocBytes {
    const OFFSET: usize = size_of::<TypeId>();
    unsafe fn unaligned_type_id(&self) -> *mut TypeId {
        unsafe { self.ptr.add(self.size).cast() }
    }
    unsafe fn alloc(size: usize) -> *mut u8 {
        unsafe {
            let ptr = alloc(Layout::from_size_align_unchecked(size + Self::OFFSET, 1));
            if !ptr.is_null() {
                let bytes = Self { ptr, size };
                bytes.unaligned_type_id().write_unaligned(TypeId::of::<Self>());
            }
            ptr
        }
    }
    unsafe fn dealloc(ptr: *mut u8, size: usize) -> Result<(), Error> {
        unsafe {
            if ptr.is_null() {
                Err(Error::BytesInvalid("null".to_string()))
            } else {
                let bytes = Self { ptr, size };
                let expect = TypeId::of::<Self>();
                let actual = bytes.unaligned_type_id().read_unaligned();
                if actual != expect {
                    let msg = format!("expect:{:?} actual:{:?}", expect, actual);
                    Err(Error::BytesInvalid(msg))
                } else {
                    Ok(dealloc(ptr, Layout::from_size_align_unchecked(size + Self::OFFSET, 1)))
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __alloc(size: usize) -> *mut u8 {
    unsafe { AllocBytes::alloc(size) }
}

#[unsafe(no_mangle)]
pub extern "C" fn __dealloc(ptr: *mut u8, size: usize) -> bool {
    unsafe {
        match AllocBytes::dealloc(ptr, size) {
            Ok(_) => true,
            Err(err) => {
                print_error(format!("{} error: {}", "__dealloc", err));
                false
            }
        }
    }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C" fn __free_AbiString(ptr: AbiString) -> bool {
    unsafe {
        match ptr.free() {
            Ok(_) => true,
            Err(err) => {
                print_error(format!("{} error: {}", "__free_AbiString", err));
                false
            }
        }
    }
}
