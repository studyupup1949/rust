use std::slice;
use crate::{ACCError, Result};

#[cfg(windows)]
use std::ffi::CString;

#[cfg(windows)]
use winapi::shared::winerror::ERROR_FILE_NOT_FOUND;
#[cfg(windows)]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(windows)]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use winapi::um::memoryapi::{MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_READ};
#[cfg(windows)]
use winapi::um::winnt::{GENERIC_READ, HANDLE};

/// Low-level shared memory reader for Windows named shared memory objects.
pub struct SharedMemoryReader {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(not(windows))]
    _handle: (),
    ptr: *mut u8,
    size: usize,
    name: String,
}

impl SharedMemoryReader {
    /// Create a new shared memory reader for the given named memory object.
    #[cfg(windows)]
    pub fn new(name: &str, size: usize) -> Result<Self> {
        let c_name = CString::new(name).map_err(|_| {
            ACCError::SharedMemoryOpen(format!("Invalid memory name: {}", name))
        })?;

        // Open the shared memory object
        let handle = unsafe {
            OpenFileMappingA(
                GENERIC_READ,
                0, // Don't inherit handle
                c_name.as_ptr(),
            )
        };

        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { GetLastError() };
            if error_code == ERROR_FILE_NOT_FOUND {
                return Err(ACCError::SharedMemoryNotAvailable);
            }
            return Err(ACCError::SharedMemoryOpen(format!(
                "Failed to open shared memory '{}', error code: {}",
                name, error_code
            )));
        }

        // Map the shared memory into our address space
        let ptr = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size) } as *mut u8;

        if ptr.is_null() {
            unsafe { CloseHandle(handle) };
            let error_code = unsafe { GetLastError() };
            return Err(ACCError::SharedMemoryMap(format!(
                "Failed to map shared memory '{}', error code: {}",
                name, error_code
            )));
        }

        Ok(Self {
            handle,
            ptr,
            size,
            name: name.to_string(),
        })
    }

    /// Create a new shared memory reader (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(name: &str, _size: usize) -> Result<Self> {
        Err(ACCError::SharedMemoryNotAvailable)
    }

    /// Get the raw pointer to the shared memory
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get the size of the shared memory region
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get a slice view of the shared memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.size) }
    }

    /// Read data from a specific offset
    pub fn read_at<T: Copy>(&self, offset: usize) -> Result<T> {
        if offset + std::mem::size_of::<T>() > self.size {
            return Err(ACCError::InvalidData(format!(
                "Read beyond buffer bounds: offset {} + size {} > buffer size {}",
                offset,
                std::mem::size_of::<T>(),
                self.size
            )));
        }

        unsafe {
            let ptr = self.ptr.add(offset) as *const T;
            Ok(ptr.read_unaligned())
        }
    }

    /// Read an array of data from a specific offset
    pub fn read_array_at<T: Copy>(&self, offset: usize, count: usize) -> Result<Vec<T>> {
        let total_size = count * std::mem::size_of::<T>();
        if offset + total_size > self.size {
            return Err(ACCError::InvalidData(format!(
                "Read beyond buffer bounds: offset {} + size {} > buffer size {}",
                offset, total_size, self.size
            )));
        }

        let mut result = Vec::with_capacity(count);
        unsafe {
            let ptr = self.ptr.add(offset) as *const T;
            for i in 0..count {
                result.push(ptr.add(i).read_unaligned());
            }
        }
        Ok(result)
    }

    /// Read a UTF-16 string from a specific offset
    pub fn read_utf16_string_at(&self, offset: usize, char_count: usize) -> Result<String> {
        let byte_count = char_count * 2; // UTF-16 uses 2 bytes per character
        if offset + byte_count > self.size {
            return Err(ACCError::InvalidData(format!(
                "String read beyond buffer bounds: offset {} + size {} > buffer size {}",
                offset, byte_count, self.size
            )));
        }

        unsafe {
            let ptr = self.ptr.add(offset) as *const u16;
            let slice = slice::from_raw_parts(ptr, char_count);
            
            // Find the null terminator
            let mut len = char_count;
            for (i, &ch) in slice.iter().enumerate() {
                if ch == 0 {
                    len = i;
                    break;
                }
            }

            String::from_utf16(&slice[..len])
                .map_err(|_| ACCError::InvalidData("Invalid UTF-16 string".to_string()))
        }
    }

    /// Get the name of this shared memory object
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for SharedMemoryReader {
    #[cfg(windows)]
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                UnmapViewOfFile(self.ptr as *const _);
            }
            if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
        }
    }

    #[cfg(not(windows))]
    fn drop(&mut self) {
        // Nothing to do on non-Windows platforms
    }
}

unsafe impl Send for SharedMemoryReader {}
unsafe impl Sync for SharedMemoryReader {}