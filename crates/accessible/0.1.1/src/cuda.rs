//! CUDA pointer probes gated behind the `cuda` feature.

use crate::CheckError;
use cust_raw::cudaError_enum::{CUDA_ERROR_NOT_INITIALIZED, CUDA_SUCCESS};
use cust_raw::CUmemorytype_enum::{CU_MEMORYTYPE_DEVICE, CU_MEMORYTYPE_UNIFIED};
use cust_raw::CUpointer_attribute_enum::{
    CU_POINTER_ATTRIBUTE_DEVICE_ORDINAL, CU_POINTER_ATTRIBUTE_MEMORY_TYPE,
};
use cust_raw::{cuPointerGetAttribute, CUdeviceptr, CUmemorytype, CUpointer_attribute, CUresult};
use std::ffi::c_void;
use std::io::{self, ErrorKind};
use std::mem;

/// Performs a best-effort check that the device memory region `[ptr, ptr + len)` is
/// associated with `device_ordinal`.
///
/// The probe touches only the first and last byte to keep the surface similar to the CPU
/// helpers. `len == 0` succeeds. On platforms without a CUDA driver the function returns
/// [`CheckError::Unsupported`].
pub fn probe_readable(ptr: *const u8, len: usize, device_ordinal: i32) -> Result<(), CheckError> {
    if len == 0 {
        return Ok(());
    }
    if ptr.is_null() {
        return Err(CheckError::NullPointer);
    }

    let end_ptr = if len == 1 {
        None
    } else {
        let end_addr = (ptr as usize)
            .checked_add(len - 1)
            .ok_or(CheckError::LengthOverflow)?;
        Some(end_addr as *const u8)
    };

    probe_single(ptr, device_ordinal)?;
    if let Some(end) = end_ptr {
        probe_single(end, device_ordinal)?;
    }

    Ok(())
}

/// Probes whether `ptr` refers to a readable `T` value on `device_ordinal`.
pub fn probe_readable_value<T>(ptr: *const T, device_ordinal: i32) -> Result<(), CheckError> {
    let size = mem::size_of::<T>();
    if size == 0 {
        return Ok(());
    }
    probe_readable(ptr.cast(), size, device_ordinal)
}

/// Probes whether `count` consecutive `T` values starting at `ptr` are readable on
/// `device_ordinal`.
pub fn probe_readable_range<T>(
    ptr: *const T,
    count: usize,
    device_ordinal: i32,
) -> Result<(), CheckError> {
    if count == 0 {
        return Ok(());
    }
    let size = mem::size_of::<T>();
    if size == 0 {
        return Ok(());
    }
    let byte_len = size.checked_mul(count).ok_or(CheckError::LengthOverflow)?;
    probe_readable(ptr.cast(), byte_len, device_ordinal)
}

/// Probes whether the provided slice resides in readable device memory on `device_ordinal`.
pub fn probe_readable_slice<T>(slice: &[T], device_ordinal: i32) -> Result<(), CheckError> {
    probe_readable_range(slice.as_ptr(), slice.len(), device_ordinal)
}

fn probe_single(ptr: *const u8, device_ordinal: i32) -> Result<(), CheckError> {
    let device_ptr = ptr as CUdeviceptr;
    let mem_type: CUmemorytype = query_attribute(
        device_ptr,
        CU_POINTER_ATTRIBUTE_MEMORY_TYPE,
        "CU_POINTER_ATTRIBUTE_MEMORY_TYPE",
    )?;

    if mem_type == CU_MEMORYTYPE_DEVICE {
        let ordinal: i32 = query_attribute(
            device_ptr,
            CU_POINTER_ATTRIBUTE_DEVICE_ORDINAL,
            "CU_POINTER_ATTRIBUTE_DEVICE_ORDINAL",
        )?;

        if ordinal != device_ordinal {
            return Err(CheckError::Unreadable(io::Error::new(
                ErrorKind::Other,
                format!("pointer is associated with device {ordinal}, expected {device_ordinal}"),
            )));
        }
        return Ok(());
    }

    if mem_type == CU_MEMORYTYPE_UNIFIED {
        return Ok(());
    }

    Err(CheckError::Unreadable(io::Error::new(
        ErrorKind::Other,
        format!(
            "memory type code {} is not accessible from CUDA device {device_ordinal}",
            mem_type as i32
        ),
    )))
}

fn query_attribute<T: Copy>(
    ptr: CUdeviceptr,
    attribute: CUpointer_attribute,
    attr_name: &str,
) -> Result<T, CheckError> {
    let mut value = mem::MaybeUninit::<T>::uninit();
    let result: CUresult =
        unsafe { cuPointerGetAttribute(value.as_mut_ptr() as *mut c_void, attribute, ptr) };

    match result {
        CUDA_SUCCESS => Ok(unsafe { value.assume_init() }),
        CUDA_ERROR_NOT_INITIALIZED => Err(CheckError::Unsupported),
        other => Err(CheckError::Unreadable(io::Error::new(
            ErrorKind::Other,
            format!("cuPointerGetAttribute({attr_name}) failed with CUDA error code {other:?}"),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cust_raw::cudaError_enum::{CUDA_ERROR_NO_DEVICE, CUDA_SUCCESS};
    use cust_raw::{
        cuCtxCreate_v2, cuCtxDestroy_v2, cuDeviceGet, cuInit, cuMemAllocHost_v2, cuMemAllocManaged,
        cuMemAlloc_v2, cuMemFreeHost, cuMemFree_v2, CUcontext, CUdevice, CUdeviceptr,
    };
    use std::ptr;

    #[test]
    fn zero_length_is_ok() {
        assert!(probe_readable(ptr::null(), 0, 0).is_ok());
    }

    #[test]
    fn rejects_null_pointer() {
        assert!(matches!(
            probe_readable(ptr::null(), 1, 0),
            Err(CheckError::NullPointer)
        ));
    }

    #[test]
    fn device_memory_is_readable() {
        let Some(ctx) = init_cuda_fixture() else {
            return;
        };

        let Some(buffer) = DeviceBuffer::allocate(16) else {
            return;
        };

        assert!(probe_readable(buffer.ptr as *const u8, 8, ctx.ordinal).is_ok());
        assert!(probe_readable_range(buffer.ptr as *const u32, 2, ctx.ordinal).is_ok());
    }

    #[test]
    fn mismatched_device_reports_error() {
        let Some(ctx) = init_cuda_fixture() else {
            return;
        };

        let Some(buffer) = DeviceBuffer::allocate(8) else {
            return;
        };

        assert!(matches!(
            probe_readable(buffer.ptr as *const u8, 4, ctx.ordinal.saturating_add(1)),
            Err(CheckError::Unreadable(_))
        ));
    }

    #[test]
    fn host_memory_is_rejected() {
        let Some(ctx) = init_cuda_fixture() else {
            return;
        };

        let Some(host) = HostBuffer::allocate(64) else {
            return;
        };

        assert!(matches!(
            probe_readable(host.ptr as *const u8, 4, ctx.ordinal),
            Err(CheckError::Unreadable(_))
        ));
        assert!(crate::probe_readable(host.ptr as *const u8, 4).is_ok());
    }

    #[test]
    fn unified_memory_is_allowed() {
        let Some(ctx) = init_cuda_fixture() else {
            return;
        };

        let Some(buffer) = UnifiedBuffer::allocate(32) else {
            return;
        };

        assert!(probe_readable(buffer.ptr as *const u8, 16, ctx.ordinal).is_ok());
        assert!(crate::probe_readable(buffer.ptr as *const u8, 16).is_ok());
    }

    struct CudaFixture {
        context: CUcontext,
        ordinal: i32,
    }

    impl Drop for CudaFixture {
        fn drop(&mut self) {
            unsafe {
                if !self.context.is_null() {
                    let _ = cuCtxDestroy_v2(self.context);
                }
            }
        }
    }

    struct DeviceBuffer {
        ptr: CUdeviceptr,
    }

    impl DeviceBuffer {
        fn allocate(bytes: usize) -> Option<Self> {
            if bytes == 0 {
                return Some(DeviceBuffer { ptr: 0 });
            }

            let mut ptr: CUdeviceptr = 0;
            let result = unsafe { cuMemAlloc_v2(&mut ptr as *mut CUdeviceptr, bytes) };
            if result != CUDA_SUCCESS {
                eprintln!("cuMemAlloc_v2 failed: {result:?}");
                return None;
            }
            Some(DeviceBuffer { ptr })
        }
    }

    impl Drop for DeviceBuffer {
        fn drop(&mut self) {
            if self.ptr != 0 {
                unsafe {
                    let _ = cuMemFree_v2(self.ptr);
                }
            }
        }
    }

    struct UnifiedBuffer {
        ptr: CUdeviceptr,
    }

    impl UnifiedBuffer {
        fn allocate(bytes: usize) -> Option<Self> {
            if bytes == 0 {
                return Some(UnifiedBuffer { ptr: 0 });
            }
            let mut ptr: CUdeviceptr = 0;
            let result = unsafe { cuMemAllocManaged(&mut ptr as *mut CUdeviceptr, bytes, 0) };
            if result != CUDA_SUCCESS {
                eprintln!("cuMemAllocManaged failed: {result:?}");
                return None;
            }
            Some(UnifiedBuffer { ptr })
        }
    }

    impl Drop for UnifiedBuffer {
        fn drop(&mut self) {
            if self.ptr != 0 {
                unsafe {
                    let _ = cuMemFree_v2(self.ptr);
                }
            }
        }
    }

    struct HostBuffer {
        ptr: *mut c_void,
    }

    impl HostBuffer {
        fn allocate(bytes: usize) -> Option<Self> {
            if bytes == 0 {
                return Some(HostBuffer {
                    ptr: ptr::null_mut(),
                });
            }
            let mut ptr: *mut c_void = ptr::null_mut();
            let result = unsafe { cuMemAllocHost_v2(&mut ptr as *mut *mut c_void, bytes) };
            if result != CUDA_SUCCESS {
                eprintln!("cuMemAllocHost_v2 failed: {result:?}");
                return None;
            }
            Some(HostBuffer { ptr })
        }
    }

    impl Drop for HostBuffer {
        fn drop(&mut self) {
            if !self.ptr.is_null() {
                unsafe {
                    let _ = cuMemFreeHost(self.ptr);
                }
            }
        }
    }

    fn init_cuda_fixture() -> Option<CudaFixture> {
        unsafe {
            match cuInit(0) {
                CUDA_SUCCESS => {}
                CUDA_ERROR_NO_DEVICE => return None,
                err => {
                    eprintln!("cuInit failed: {err:?}");
                    return None;
                }
            }

            let ordinal = 0;
            let mut device: CUdevice = 0;
            match cuDeviceGet(&mut device as *mut CUdevice, ordinal) {
                CUDA_SUCCESS => {}
                err => {
                    eprintln!("cuDeviceGet failed: {err:?}");
                    return None;
                }
            }

            let mut context: CUcontext = ptr::null_mut();
            match cuCtxCreate_v2(&mut context as *mut CUcontext, 0, device) {
                CUDA_SUCCESS => {}
                err => {
                    eprintln!("cuCtxCreate_v2 failed: {err:?}");
                    return None;
                }
            }

            Some(CudaFixture { context, ordinal })
        }
    }
}
