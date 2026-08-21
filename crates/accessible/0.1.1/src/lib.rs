//! Best-effort memory readability probes that avoid trapping on invalid pointers.
//!
//! The functions in this crate are designed for FFI boundaries where you want to sanity-check
//! user-provided pointers. They rely on platform facilities that report faults as recoverable
//! errors instead of signalling `SIGSEGV`. All checks are heuristics by nature—another thread can
//! still revoke access after a probe succeeds—so treat the results as hints rather than hard
//! guarantees.

use std::error::Error;
use std::fmt;
use std::io;
use std::mem;

const PROBE_BYTES: usize = 1;

/// Errors returned by the probe helpers.
#[derive(Debug)]
pub enum CheckError {
    /// A null pointer was supplied with a non-zero length.
    NullPointer,
    /// The computed pointer range overflowed the address space.
    LengthOverflow,
    /// The current platform does not provide a supported non-trapping read primitive.
    Unsupported,
    /// The platform probe call reported the memory as unreadable.
    Unreadable(io::Error),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::NullPointer => write!(f, "null pointer provided"),
            CheckError::LengthOverflow => write!(f, "pointer range overflowed usize"),
            CheckError::Unsupported => write!(f, "platform does not support non-trapping probe"),
            CheckError::Unreadable(err) => write!(f, "memory read failed: {err}"),
        }
    }
}

impl Error for CheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CheckError::Unreadable(err) => Some(err),
            _ => None,
        }
    }
}

/// Performs a best-effort check that the region `[ptr, ptr + len)` is readable without trapping.
///
/// The probe touches only the first and last byte. `len == 0` is treated as readable so callers
/// can forward zero-length slices without special casing. On unsupported platforms the function
/// returns [`CheckError::Unsupported`].
///
/// ```
/// use accessible::probe_readable;
///
/// let buf = [0u8; 4];
/// assert!(probe_readable(buf.as_ptr(), buf.len()).is_ok());
/// ```
pub fn probe_readable(ptr: *const u8, len: usize) -> Result<(), CheckError> {
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

    platform::probe_single(ptr)?;
    if let Some(end) = end_ptr {
        platform::probe_single(end)?;
    }

    Ok(())
}

/// Probes whether `ptr` refers to a readable `T` value.
///
/// Zero-sized types always succeed.
pub fn probe_readable_value<T>(ptr: *const T) -> Result<(), CheckError> {
    let size = mem::size_of::<T>();
    if size == 0 {
        return Ok(());
    }
    probe_readable(ptr.cast(), size)
}

/// Probes whether `count` consecutive `T` values starting at `ptr` are readable.
///
/// Empty ranges and zero-sized types automatically succeed.
pub fn probe_readable_range<T>(ptr: *const T, count: usize) -> Result<(), CheckError> {
    if count == 0 {
        return Ok(());
    }
    let size = mem::size_of::<T>();
    if size == 0 {
        return Ok(());
    }
    let byte_len = size.checked_mul(count).ok_or(CheckError::LengthOverflow)?;
    probe_readable(ptr.cast(), byte_len)
}

/// Probes whether the provided slice resides in readable memory.
pub fn probe_readable_slice<T>(slice: &[T]) -> Result<(), CheckError> {
    probe_readable_range(slice.as_ptr(), slice.len())
}

#[cfg(feature = "cuda")]
pub mod cuda;

mod platform {
    use super::{CheckError, PROBE_BYTES};

    #[cfg(target_os = "linux")]
    pub(super) fn probe_single(ptr: *const u8) -> Result<(), CheckError> {
        match try_process_vm_readv(ptr) {
            Ok(()) => Ok(()),
            Err(CheckError::Unsupported) => try_proc_self_mem(ptr),
            Err(other) => Err(other),
        }
    }

    #[cfg(target_os = "linux")]
    fn try_process_vm_readv(ptr: *const u8) -> Result<(), CheckError> {
        use libc::iovec;
        use std::ffi::c_void;
        use std::io::{self, ErrorKind};

        let pid = unsafe { libc::getpid() };
        let mut buffer = [0u8; PROBE_BYTES];
        let local = [iovec {
            iov_base: buffer.as_mut_ptr() as *mut c_void,
            iov_len: buffer.len(),
        }];
        let remote = [iovec {
            iov_base: ptr as *mut c_void,
            iov_len: buffer.len(),
        }];

        let read = unsafe {
            libc::process_vm_readv(
                pid,
                local.as_ptr(),
                local.len() as libc::c_ulong,
                remote.as_ptr(),
                remote.len() as libc::c_ulong,
                0 as libc::c_ulong,
            )
        };

        if read == -1 {
            let err = io::Error::last_os_error();
            return match err.raw_os_error() {
                Some(code) if code == libc::ENOSYS => Err(CheckError::Unsupported),
                _ => Err(CheckError::Unreadable(err)),
            };
        }

        if read as usize != buffer.len() {
            return Err(CheckError::Unreadable(io::Error::new(
                ErrorKind::Other,
                "short read from process_vm_readv",
            )));
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn try_proc_self_mem(ptr: *const u8) -> Result<(), CheckError> {
        use std::fs::File;
        use std::io::{self, ErrorKind};
        use std::os::unix::fs::FileExt;

        let mut buffer = [0u8; PROBE_BYTES];
        let file = File::open("/proc/self/mem").map_err(CheckError::Unreadable)?;
        let bytes_read = file
            .read_at(&mut buffer, ptr as u64)
            .map_err(CheckError::Unreadable)?;

        if bytes_read == buffer.len() {
            Ok(())
        } else {
            Err(CheckError::Unreadable(io::Error::new(
                ErrorKind::UnexpectedEof,
                "short read from /proc/self/mem",
            )))
        }
    }

    #[cfg(target_os = "macos")]
    pub(super) fn probe_single(ptr: *const u8) -> Result<(), CheckError> {
        use mach2::kern_return::{kern_return_t, KERN_SUCCESS};
        use mach2::mach_types::vm_task_entry_t;
        use mach2::traps::mach_task_self;
        use mach2::vm::mach_vm_read_overwrite;
        use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
        use std::io::{self, ErrorKind};

        let mut buffer = [0u8; PROBE_BYTES];
        let mut out_size: mach_vm_size_t = 0;
        let task: vm_task_entry_t = unsafe { mach_task_self() };
        let result: kern_return_t = unsafe {
            mach_vm_read_overwrite(
                task,
                ptr as mach_vm_address_t,
                PROBE_BYTES as mach_vm_size_t,
                buffer.as_mut_ptr() as mach_vm_address_t,
                &mut out_size,
            )
        };

        if result == KERN_SUCCESS {
            if out_size as usize == buffer.len() {
                Ok(())
            } else {
                Err(CheckError::Unreadable(io::Error::new(
                    ErrorKind::Other,
                    "short read from mach_vm_read_overwrite",
                )))
            }
        } else {
            Err(CheckError::Unreadable(io::Error::new(
                ErrorKind::Other,
                format!("mach error {result}"),
            )))
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn probe_single(ptr: *const u8) -> Result<(), CheckError> {
        use std::ffi::c_void;
        use std::io::{self, ErrorKind};
        use windows_sys::Win32::Foundation::GetLastError;
        use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        let process = unsafe { GetCurrentProcess() };
        let mut buffer = [0u8; PROBE_BYTES];
        let mut bytes_read: usize = 0;
        let ok = unsafe {
            ReadProcessMemory(
                process,
                ptr as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                &mut bytes_read,
            )
        };

        if ok == 0 {
            let err_code = unsafe { GetLastError() };
            return Err(CheckError::Unreadable(io::Error::from_raw_os_error(
                err_code as i32,
            )));
        }

        if bytes_read == buffer.len() {
            Ok(())
        } else {
            Err(CheckError::Unreadable(io::Error::new(
                ErrorKind::Other,
                "short read from ReadProcessMemory",
            )))
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub(super) fn probe_single(_: *const u8) -> Result<(), CheckError> {
        Err(CheckError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_length_is_ok() {
        assert!(probe_readable(std::ptr::null(), 0).is_ok());
    }

    #[test]
    fn rejects_null_pointer() {
        assert!(matches!(
            probe_readable(std::ptr::null(), 1),
            Err(CheckError::NullPointer)
        ));
    }

    #[test]
    fn detects_length_overflow() {
        let ptr = usize::MAX as *const u8;
        assert!(matches!(
            probe_readable(ptr, 2),
            Err(CheckError::LengthOverflow)
        ));
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn stack_memory_is_readable() {
        let data = [1u8, 2, 3, 4];
        assert!(probe_readable(data.as_ptr(), data.len()).is_ok());
        assert!(probe_readable_slice(&data).is_ok());
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn pointer_wrappers_work() {
        let values = [1u64, 2, 3];
        assert!(probe_readable_value(values.as_ptr()).is_ok());
        assert!(probe_readable_range(values.as_ptr(), values.len()).is_ok());
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn bogus_pointer_is_unreadable() {
        let ptr = 0x1 as *const u8;
        assert!(matches!(
            probe_readable(ptr, 1),
            Err(CheckError::Unreadable(_))
        ));
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    #[test]
    fn unsupported_platform_is_reported() {
        let data = [0u8; 4];
        assert!(matches!(
            probe_readable(data.as_ptr(), data.len()),
            Err(CheckError::Unsupported)
        ));
    }

    #[test]
    fn zst_range_is_ok() {
        #[derive(Copy, Clone)]
        struct Zst;
        let values = [Zst, Zst];
        assert!(probe_readable_slice(&values).is_ok());
        assert!(probe_readable_range(values.as_ptr(), values.len()).is_ok());
    }
}
