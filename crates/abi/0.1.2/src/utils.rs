use crate::error::Error;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::ptr::slice_from_raw_parts_mut;
use std::slice::from_raw_parts;

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
    AbiPtr::new(value)
}

pub trait CheckTypeId: 'static {
    fn check(self: &Self) -> bool;
}
#[repr(C)]
pub struct FatAbiString {
    ptr: *mut u8,
    len: usize,
    _type: TypeId,
}
impl FatAbiString {
    pub fn new(str: String) -> Self {
        let len = str.len();
        let ptr = Box::into_raw(str.into_boxed_str()) as _;
        Self {
            ptr,
            len,
            _type: TypeId::of::<Self>(),
        }
    }
    pub unsafe fn as_str(&self) -> Result<&str, Error> {
        if self.ptr.is_null() {
            Err(Error::StringNull)
        } else {
            let slice = unsafe { from_raw_parts(self.ptr, self.len) };
            std::str::from_utf8(slice).map_err(Error::from)
        }
    }
    pub unsafe fn free(self) -> Result<String, Error> {
        if self.ptr.is_null() {
            Err(Error::StringNull)
        } else {
            let slice = slice_from_raw_parts_mut(self.ptr, self.len);
            let str = unsafe { std::str::from_boxed_utf8_unchecked(Box::from_raw(slice)) };
            Ok(str.to_string())
        }
    }
}
impl Pointer for FatAbiString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pointer")
            .field("addr", &PointerAsDebug(self.ptr))
            .field("metadata", &self.len)
            .finish()
    }
}
impl CheckTypeId for FatAbiString {
    fn check(self: &Self) -> bool {
        self._type == TypeId::of::<Self>()
    }
}

#[repr(C)]
pub struct AbiString(AbiPtr<FatAbiString>);
impl AbiString {
    pub fn null() -> Self {
        Self(AbiPtr::null())
    }
    pub fn new(str: String) -> Self {
        AbiString(AbiPtr::new(FatAbiString::new(str)))
    }
    pub unsafe fn as_str(&self) -> Result<&str, Error> {
        //ignore typeId check
        unsafe { (&*self.0.ptr).as_str() }
    }
    pub unsafe fn free(self) -> Result<String, Error> {
        unsafe { self.0.free()?.free() }
    }
}
impl Clone for AbiString {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl Copy for AbiString {}
impl Pointer for AbiString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Pointer");
        debug.field("addr", &PointerAsDebug(self.0));
        let ptr = self.0.ptr;
        if !ptr.is_null() {
            debug.field("metadata", &PointerAsDebug(ptr));
        }
        debug.finish()
    }
}

#[repr(C)]
pub struct AbiPtr<T = ()> {
    ptr: *mut T,
}
impl<T: CheckTypeId> AbiPtr<T> {
    pub fn null() -> Self {
        Self { ptr: std::ptr::null_mut() }
    }
    pub fn new(value: T) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(value)),
        }
    }
    pub unsafe fn as_mut(&self) -> Result<&'static mut T, Error> {
        if self.ptr.is_null() {
            Err(Error::NPE)
        } else {
            let value = unsafe { &mut *self.ptr };
            if value.check() {
                Ok(value)
            } else {
                Err(Error::TypeInvalid(std::any::type_name::<T>()))
            }
        }
    }
    pub unsafe fn free(self) -> Result<Box<T>, Error> {
        unsafe {
            self.as_mut()?;
        }
        Ok(unsafe { Box::from_raw(self.ptr) })
    }
}
impl<T> Clone for AbiPtr<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}
impl<T> Copy for AbiPtr<T> {}
impl<T> Pointer for AbiPtr<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pointer")
            .field("addr", &PointerAsDebug(self.ptr))
            .field("metadata", &DisplayAsDebug(std::any::type_name::<T>()))
            .finish()
    }
}
struct DisplayAsDebug<T>(T);
impl<T: Display> Debug for DisplayAsDebug<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
struct PointerAsDebug<T>(T);
impl<T: Pointer> Debug for PointerAsDebug<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Pointer::fmt(&self.0, f)
    }
}
