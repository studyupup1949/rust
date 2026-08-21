use core::marker::PhantomData;

use alloc::vec::Vec;

use super::types::FfiAcpiBuffer;

//
const ACPI_ALLOCATE_BUFFER: usize = usize::MAX;

#[derive(Debug)]
pub(crate) struct BufferUnchangedError;

impl FfiAcpiBuffer<'static> {
    pub const EMPTY: Self = Self {
        length: 0,
        pointer: core::ptr::null_mut(),
        _p: PhantomData,
    };

    pub const ALLOCATE: Self = Self {
        length: ACPI_ALLOCATE_BUFFER,
        pointer: core::ptr::null_mut(),
        _p: PhantomData,
    };

    pub(crate) fn allocate<T, U>(f: T) -> (U, Result<Vec<u8>, BufferUnchangedError>)
    where
        T: Fn(&mut Self) -> U,
    {
        let mut b = Self::ALLOCATE;

        let u = f(&mut b);

        // Check that the buffer has been written to
        if b.length == ACPI_ALLOCATE_BUFFER {
            return (u, Err(BufferUnchangedError));
        }

        // SAFETY: This memory was allocated by ACPICA
        // This means the call went through AcpiOsAllocate, which uses the system allocator
        // Therefore, the pointer, length, capacity, and allocator are all valid to construct a Vec
        let v = unsafe { Vec::from_raw_parts(b.pointer.cast(), b.length, b.length) };

        (u, Ok(v))
    }
}

pub(crate) trait VecToBufferExt {
    fn to_buffer(&mut self) -> FfiAcpiBuffer;
}

impl<T> VecToBufferExt for Vec<T> {
    fn to_buffer(&mut self) -> FfiAcpiBuffer {
        FfiAcpiBuffer {
            length: self.len(),
            pointer: self.as_mut_slice().as_mut_ptr().cast(),
            _p: PhantomData,
        }
    }
}
