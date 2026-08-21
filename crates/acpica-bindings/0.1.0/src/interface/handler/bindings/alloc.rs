use alloc::vec::Vec;
use log::trace;

use crate::bindings::types::FfiAcpiSize;

#[export_name = "AcpiOsAllocate"]
extern "C" fn acpi_os_allocate(size: FfiAcpiSize) -> *mut ::core::ffi::c_void {
    trace!(target: "acpi_os_allocate", "Allocating {size:#x} bytes of memory");

    let mut v = Vec::<u8>::new();

    let total_allocation_size = size + core::mem::size_of::<usize>();
    let Ok(()) = v.try_reserve_exact(total_allocation_size) else {
        trace!(target: "acpi_os_allocate", "Allocation failed");
        return core::ptr::null_mut();
    };

    assert_eq!(v.capacity(), total_allocation_size);

    let ptr = v.as_mut_ptr();
    core::mem::forget(v);

    trace!(target: "acpi_os_allocate", "Allocated memory at {ptr:p}");

    // SAFETY:
    // There are no references to the Vec any more, so writing to its memory is sound.
    unsafe { core::ptr::write_unaligned(ptr.cast::<usize>(), size) }

    // SAFETY: This is adding <=8 bytes so it can't exceed the size bounds
    unsafe { ptr.byte_add(core::mem::size_of::<usize>()) }.cast()
}

#[export_name = "AcpiOsFree"]
extern "C" fn acpi_os_free(memory: *mut ::core::ffi::c_void) {
    // SAFETY: The pointer passed to ACPICA was one usize more than the actual allocated Vec,
    // so this pointer is part of the same allocation.
    let real_start = unsafe { memory.byte_sub(core::mem::size_of::<usize>()) };
    // SAFETY: A usize was written here in `allocate`, so it can be read here.
    let size = unsafe { core::ptr::read_unaligned(real_start as *const usize) };
    // SAFETY: This pointer was allocated by Vec with this size and capacity in `allocate`.
    let v = unsafe { Vec::from_raw_parts(real_start, size, size) };

    // Explicitly drop the Vec to free the memory
    drop(v);
}

// TODO: Native AllocateZeroed
