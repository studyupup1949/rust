use crate::error::Error;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::ptr::slice_from_raw_parts;

pub unsafe fn from_json<T: DeserializeOwned>(ptr: AbiString) -> Result<T, Error> {
    Ok(serde_json::from_str(unsafe { ptr.as_str()? })?)
}
pub unsafe fn from_ptr<T: CheckTypeId>(ptr: AbiPtr<T>) -> Result<&'static mut T, Error> {
    Ok(unsafe { ptr.as_mut()? })
}
pub fn into_json<T: Serialize>(value: T) -> Result<AbiString, Error> {
    Ok(AbiString::new(serde_json::to_string(&value)?))
}
pub fn into_ptr<T: CheckTypeId>(value: T) -> AbiPtr<T> {
    AbiPtr::new(Box::new(value))
}

pub trait CheckTypeId {
    fn check(self: &Self) -> bool;
}
impl CheckTypeId for String {
    fn check(self: &Self) -> bool {
        true
    }
}
#[repr(C)]
pub struct FatAbiString {
    ptr: *const u8,
    len: usize,
}
impl FatAbiString {
    pub fn null() -> Self {
        Self {
            ptr: std::ptr::null(),
            len: 0,
        }
    }
    pub fn new(str: String) -> Self {
        let len = str.len();
        let ptr = Box::into_raw(str.into_boxed_str());
        Self {
            len,
            ptr: ptr as *const u8,
        }
    }
    pub unsafe fn as_str(&self) -> Result<&str, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            let parts = slice_from_raw_parts(self.ptr, self.len);
            Ok(unsafe { &*(parts as *const str) })
        }
    }
    pub unsafe fn drop(self) -> Result<String, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            let parts = slice_from_raw_parts(self.ptr, self.len);
            let str = unsafe { Box::from_raw(parts as *mut str) };
            Ok(str.into_string())
        }
    }
}
impl Clone for FatAbiString {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
        }
    }
}
impl Copy for FatAbiString {}

#[repr(C)]
pub struct AbiString {
    ptr: *mut FatAbiString,
}
impl AbiString {
    pub fn null() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }
    pub fn new(str: String) -> Self {
        AbiString {
            ptr: Box::into_raw(Box::new(FatAbiString::new(str))),
        }
    }
    pub unsafe fn as_str(&self) -> Result<&str, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            unsafe { (&*self.ptr).as_str() }
        }
    }
    pub unsafe fn drop(self) -> Result<String, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            let ptr = unsafe { &*self.ptr as *const FatAbiString };
            let _ = unsafe { Box::from_raw(self.ptr) };
            unsafe { (&*ptr).drop() }
        }
    }
}
impl Clone for AbiString {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}
impl Copy for AbiString {}

#[repr(C)]
pub struct AbiPtr<T> {
    ptr: *const T,
}
impl<T: CheckTypeId> AbiPtr<T> {
    pub fn null() -> Self {
        Self {
            ptr: std::ptr::null(),
        }
    }
    pub fn new(value: Box<T>) -> Self {
        Self {
            ptr: Box::into_raw(value),
        }
    }
    pub unsafe fn as_mut(&self) -> Result<&'static mut T, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            Ok(unsafe { &mut *(self.ptr as *mut T) })
        }
    }
    pub unsafe fn drop(self) -> Result<Box<T>, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            let value = unsafe { Box::from_raw(self.ptr as *mut T) };
            if value.check() {
                Ok(value)
            } else {
                Err(Error::CheckInvalid)
            }
        }
    }
}
impl<T> Clone for AbiPtr<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}
impl<T> Copy for AbiPtr<T> {}
