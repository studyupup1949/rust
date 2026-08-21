#[cfg(not(all(feature = "builtin_cache", feature = "builtin_lock",)))]
use crate::types::{AcpiAllocationError, AcpiCpuFlags};

use core::ffi::c_void;

use alloc::string::String;

use crate::{
    interface::status::AcpiError,
    types::{
        tables::AcpiTableHeader, AcpiInterruptCallback, AcpiInterruptCallbackTag, AcpiIoAddress,
        AcpiMappingError, AcpiPciId, AcpiPhysicalAddress, AcpiPredefinedNames, AcpiThreadCallback,
    },
};

/// The interface between ACPICA and the host OS. Each method in this trait is mapped to an `AcpiOs...` function,
/// which will be called on the object registered with [`register_interface`].
///
/// # Optional Methods
///
/// Some methods are only present if certain features of the crate are enabled or disabled:
/// * `create_cache`, `delete_cache`, `purge_cache`, `acquire_object`, and `release_object`
///     are only present if the crate feature `builtin_cache` is disabled
/// * `create_lock`, `delete_lock`, `acquire_lock`, and `release_lock`
///     are only present if the crate feature `builtin_lock` is disabled
/// * `create_semaphore`, `delete_semaphore`, `wait_semaphore`, and `signal_semaphore`
///     are only present if the crate feature `builtin_semaphore` is disabled
///
/// # Safety
/// This trait is unsafe to implement because some functions have restrictions on their
/// implementation as well as their caller. This is indicated per method under the heading "Implementation Safety".
///
/// As well as this, it is undefined behaviour for a panic to unwind across an FFI boundary from rust code to C code.
/// Users of this library who have panic unwinding enabled are responsible for ensuring that a panic never unwinds out of a method in this trait.
/// If the OS is built with `panic=abort`, this is not an issue.
///
/// [`register_interface`]: crate::register_interface
pub unsafe trait AcpiHandler {
    /// Method called when ACPICA initialises. The default implementation of this method is a no-op.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsInitialize`
    unsafe fn initialize(&mut self) -> Result<(), AcpiError> {
        Ok(())
    }

    /// Method called when ACPICA shuts down. The default implementation of this method is a no-op
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsTerminate`
    /// * After this method is called, the object will be dropped and no other methods will be called
    unsafe fn terminate(&mut self) -> Result<(), AcpiError> {
        Ok(())
    }

    /// Gets a physical pointer to the RSDP.
    ///
    /// # Implementation Safety
    /// * The returned pointer must point to the system's RSDP.
    fn get_root_pointer(&mut self) -> AcpiPhysicalAddress;

    /// Allows the OS to specify an override for a predefined object in the ACPI namespace.
    /// The returned string will be converted to a [`CString`], so the FFI handler for this
    /// method will panic if it contains null bytes.
    ///
    /// # Safety
    /// * This function is only called from `AcpiOsPredefinedOverride`
    ///
    /// [`CString`]: alloc::ffi::CString
    #[allow(unused_variables)]
    unsafe fn predefined_override(
        &mut self,
        predefined_object: &AcpiPredefinedNames,
    ) -> Result<Option<String>, AcpiError> {
        Ok(None)
    }

    /// Allows the OS to override an ACPI table using a logical address.
    /// This method is called once on each ACPI table in the order they are listed in the DSDT/XSDT,
    /// and when tables are loaded by the `Load` AML instruction. To keep the original table, return `Ok(None)`.
    ///
    /// To override the table using a physical address instead, use [`physical_table_override`]
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsTableOverride`
    ///
    /// [`physical_table_override`]: AcpiHandler::physical_table_override
    #[allow(unused_variables)]
    unsafe fn table_override(
        &mut self,
        table: &AcpiTableHeader,
    ) -> Result<Option<AcpiTableHeader>, AcpiError> {
        Ok(None)
    }

    /// Allows the OS to override an ACPI table using a physical address.
    /// To keep the original table, return `Ok(None)`
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsPhysicalTableOverride`
    ///
    /// # Implementation Safety
    /// * The returned physical address must point to a valid new ACPI table with the returned length
    /// * The memory indicated by the returned pointer and length is now managed by ACPICA and must
    ///     not be written to while ACPICA is active
    #[allow(unused_variables)]
    unsafe fn physical_table_override(
        &mut self,
        table: &AcpiTableHeader,
    ) -> Result<Option<(AcpiPhysicalAddress, u32)>, AcpiError> {
        Ok(None)
    }

    /// Map `length` bytes of physical memory starting at `physical_address`, and return the virtual address where they have been mapped.
    ///
    /// # Safety
    /// * This function is only called from `AcpiOsMapMemory`
    /// * The memory at `physical_address` is valid for writes for `length` bytes
    ///
    /// # Implementation Safety
    /// * The memory must stay mapped until `unmap_memory` is called.
    unsafe fn map_memory(
        &mut self,
        physical_address: AcpiPhysicalAddress,
        length: usize,
    ) -> Result<*mut u8, AcpiMappingError>;

    /// Unmap `length` pages bytes of memory which were previously allocated with [`map_memory`]
    ///
    /// # Safety
    /// * This function is only called from `AcpiOsUnmapMemory`
    /// * `address` is a pointer which was previously returned from [`map_memory`]
    ///
    /// [`map_memory`]: AcpiHandler::map_memory
    unsafe fn unmap_memory(&mut self, address: *mut u8, length: usize);

    /// Translate a logical address to the physical address it's mapped to.
    ///
    /// # Return value
    /// * `Ok(Some(address))`: The translation was successful
    /// * `Ok(None)`: The translation was successful but the virtual address is not mapped
    /// * `Err(e)`: There was an error carrying out the translation
    fn get_physical_address(
        &mut self,
        logical_address: *mut u8,
    ) -> Result<Option<AcpiPhysicalAddress>, AcpiError>;

    /// Read a [`u8`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for reads
    /// * This method is only called from `AcpiOsReadMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be from memory mapped IO, the read should be volatile
    ///
    /// If you want your implementation of this method to look the same as [`read_physical_u16`], [`..._u32`], and [`..._u64`],
    /// Read the documentation for these methods before implementing this one as the size of the type makes a difference in implementing the method soundly.
    ///
    /// [`read_physical_u16`]: AcpiHandler::read_physical_u16
    /// [`..._u32`]: AcpiHandler::read_physical_u32
    /// [`..._u64`]: AcpiHandler::read_physical_u64
    unsafe fn read_physical_u8(&mut self, address: AcpiPhysicalAddress) -> Result<u8, AcpiError>;

    /// Read a [`u16`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for reads
    /// * This method is only called from `AcpiOsReadMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be from memory mapped IO, the read should be volatile
    /// * The physical address may not be 2 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_load`] method.
    /// Otherwise, it is possible to read the data as a `[u8; 2]` and then transmute it into a [`u16`].
    unsafe fn read_physical_u16(&mut self, address: AcpiPhysicalAddress) -> Result<u16, AcpiError>;

    /// Read a [`u32`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for reads
    /// * This method is only called from `AcpiOsReadMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be from memory mapped IO, the read should be volatile
    /// * The physical address may not be 4 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_load`] method.
    /// Otherwise, it is possible to read the data as a `[u8; 4]` and then transmute it into a [`u32`].
    unsafe fn read_physical_u32(&mut self, address: AcpiPhysicalAddress) -> Result<u32, AcpiError>;

    /// Read a [`u64`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for reads
    /// * This method is only called from `AcpiOsReadMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be from memory mapped IO, the read should be volatile
    /// * The physical address may not be 8 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_load`] method.
    /// Otherwise, it is possible to read the data as a `[u8; 8]` and then transmute it into a [`u64`].
    unsafe fn read_physical_u64(&mut self, address: AcpiPhysicalAddress) -> Result<u64, AcpiError>;

    /// Read a [`u8`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for writes
    /// * This method is only called from `AcpiOsWriteMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be to memory mapped IO, the write should be volatile
    ///
    /// If you want your implementation of this method to look the same as [`write_physical_u16`], [`..._u32`], and [`..._u64`],
    /// Read the documentation for these methods before implementing this one as the size of the type makes a difference in implementing the method soundly.
    ///
    /// [`write_physical_u16`]: AcpiHandler::write_physical_u16
    /// [`..._u32`]: AcpiHandler::write_physical_u32
    /// [`..._u64`]: AcpiHandler::write_physical_u64
    unsafe fn write_physical_u8(
        &mut self,
        address: AcpiPhysicalAddress,
        value: u8,
    ) -> Result<(), AcpiError>;

    /// Read a [`u16`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for writes
    /// * This method is only called from `AcpiOsWriteMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be to memory mapped IO, the write should be volatile
    /// * The physical address may not be 2 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_store`] method.
    /// Otherwise, it is possible to transmute the data into a `[u8; 2]` before writing it.
    unsafe fn write_physical_u16(
        &mut self,
        address: AcpiPhysicalAddress,
        value: u16,
    ) -> Result<(), AcpiError>;

    /// Read a [`u32`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for writes
    /// * This method is only called from `AcpiOsWriteMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be to memory mapped IO, the write should be volatile
    /// * The physical address may not be 4 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_store`] method.
    /// Otherwise, it is possible to transmute the data into a `[u8; 4]` before writing it.
    unsafe fn write_physical_u32(
        &mut self,
        address: AcpiPhysicalAddress,
        value: u32,
    ) -> Result<(), AcpiError>;

    /// Read a [`u64`] from the given physical address.
    ///
    /// # Safety
    /// * The given physical address is valid for writes
    /// * This method is only called from `AcpiOsWriteMemory`
    ///
    /// # Implementation Safety
    /// * As this read could be to memory mapped IO, the write should be volatile
    /// * The physical address may not be 8 byte aligned, so the read should be unaligned
    ///
    /// These requirements can be difficult to satisfy at the same time.
    /// If you are alright with using unstable compiler intrinsics, the [`core::intrinsics::unaligned_volatile_store`] method.
    /// Otherwise, it is possible to transmute the data into a `[u8; 8]` before writing it.
    unsafe fn write_physical_u64(
        &mut self,
        address: AcpiPhysicalAddress,
        value: u64,
    ) -> Result<(), AcpiError>;

    /// Check whether `pointer` is valid for reads of `length` bytes.
    /// This is only in terms of the memory being mapped with the right permissions and valid, not in terms of rust's ownership rules.
    ///
    /// # Return Value
    /// * `true` if the pointer is valid for `length` bytes of reads
    /// * `false` if the pointer is not valid
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadable`
    unsafe fn readable(&mut self, pointer: *mut c_void, length: usize) -> bool;

    /// Check whether `pointer` is valid for writes of `length` bytes.
    /// This is only in terms of the memory being mapped with the right permissions and valid, not in terms of rust's ownership rules.
    ///
    /// # Return Value
    /// * `true` if the pointer is valid for `length` bytes of writes
    /// * `false` if the pointer is not valid
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWritable`
    unsafe fn writable(&mut self, pointer: *mut c_void, length: usize) -> bool;

    /// Register the given `callback` to run in the interrupt handler for the given `interrupt_number`
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsInstallInterruptHandler`
    unsafe fn install_interrupt_handler(
        &mut self,
        interrupt_number: u32,
        callback: AcpiInterruptCallback,
    ) -> Result<(), AcpiError>;

    /// Remove an interrupt handler which was previously registered with [`install_interrupt_handler`].
    /// The passed `tag` should be compared to each registered handler using [`is_tag`], and whichever handler returns `true` should be removed.
    /// If no handler is found, [`AcpiError::NotExist`] should be returned.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsRemoveInterruptHandler`
    ///
    /// # Implementation Safety
    /// * If the handler is found, it must be removed and not called again and [`Ok`] returned.
    /// * If the handler is not found, [`AcpiError::NotExist`] must be returned.
    ///
    /// [`install_interrupt_handler`]: AcpiHandler::install_interrupt_handler
    /// [`is_tag`]: AcpiInterruptCallback::is_tag
    unsafe fn remove_interrupt_handler(
        &mut self,
        interrupt_number: u32,
        tag: AcpiInterruptCallbackTag,
    ) -> Result<(), AcpiError>;

    /// Gets the thread ID of the kernel thread this method is called from.
    ///
    /// # Implementation safety
    /// * The returned thread ID must be and must be unique to the executing thread
    /// * The thread ID may not be 0 and may not be equal to [`u64::MAX`]
    fn get_thread_id(&mut self) -> u64;

    /// Run the callback in a new kernel thread. The [`call`] method of the given `callback` must be run, and then the kernel thread should be destroyed.
    /// The OS should keep track of which kernel threads were spawned using this method so that [`wait_for_events`] can be implemented correctly.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsExecute`
    ///
    /// # Return value
    /// * `Ok(())`: The thread is queued and ready to execute
    /// * `Err(e)`: There was an error creating the thread
    ///
    /// [`call`]: AcpiThreadCallback::call
    /// [`wait_for_events`]: AcpiHandler::wait_for_events
    unsafe fn execute(
        &mut self,
        // callback_type: AcpiExecuteType,
        callback: AcpiThreadCallback,
    ) -> Result<(), AcpiError>;

    /// Waits for all tasks run with [`execute`] to complete before returning.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWaitEventsComplete`
    ///
    /// # Implementation Safety
    /// * This method must not return until all
    ///
    /// [`execute`]: AcpiHandler::execute
    unsafe fn wait_for_events(&mut self);

    /// Sleep the current kernel thread for the given number of milliseconds
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsSleep`
    ///
    /// # Implementation Safety
    /// * This method must not return until the given number of milliseconds has elapsed. The OS should attempt not to overshoot the target time by too much.
    unsafe fn sleep(&mut self, millis: usize);

    /// Loop for the given number of microseconds, without sleeping the kernel thread
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsStall`
    ///
    /// # Implementation Safety
    /// * This method must not return until the given number of microseconds has elapsed. The OS should attempt not to overshoot the target time by too much.
    unsafe fn stall(&mut self, micros: usize);

    /// Print a message to the kernel's output.
    ///
    /// Multiple calls to `printf` may be used to print a single line of output, and ACPICA will write a newline character at the end of each line.
    /// For this reason, the OS should not add its own newline characters or this could break formatting.
    /// If your kernel has a macro which behaves like the standard `print!` macro, the implementation of this method can be as simple as
    ///
    /// ```ignore
    /// fn printf(&mut self, message: core::fmt::Arguments) {
    ///     print!("{message}");
    /// }
    /// ```
    fn printf(&mut self, message: core::fmt::Arguments);

    /// Read a [`u8`] from the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPort`
    unsafe fn read_port_u8(&mut self, address: AcpiIoAddress) -> Result<u8, AcpiError>;

    /// Read a [`u16`] from the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPort`
    unsafe fn read_port_u16(&mut self, address: AcpiIoAddress) -> Result<u16, AcpiError>;

    /// Read a [`u32`] from the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPort`
    unsafe fn read_port_u32(&mut self, address: AcpiIoAddress) -> Result<u32, AcpiError>;

    /// Write a [`u8`] to the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWritePort`
    unsafe fn write_port_u8(&mut self, address: AcpiIoAddress, value: u8) -> Result<(), AcpiError>;

    /// Write a [`u16`] to the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWritePort`
    unsafe fn write_port_u16(
        &mut self,
        address: AcpiIoAddress,
        value: u16,
    ) -> Result<(), AcpiError>;

    /// Write a [`u32`] to the given port
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWritePort`
    unsafe fn write_port_u32(
        &mut self,
        address: AcpiIoAddress,
        value: u32,
    ) -> Result<(), AcpiError>;

    /// Called just before the system enters a sleep state.
    /// This method allows the OS to do any final processing before entering the new state.
    /// The default implementation is a no-op.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsEnterSleep`
    ///
    // TODO: Figure out what reg_a and reg_b are and add docs
    #[allow(unused_variables)]
    unsafe fn enter_sleep(&mut self, state: u8, reg_a: u32, reg_b: u32) -> Result<(), AcpiError> {
        Ok(())
    }

    /// Get the value of the system timer in 100 nanosecond units
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsGetTimer`
    ///
    /// # Implementation Safety
    /// * The timer must not decrease i.e. later calls to this function must return a greater value
    // TODO: There might be more safety conditions
    unsafe fn get_timer(&mut self) -> u64;

    /// Read a [`u8`] from the configuration space of the given PCI ID and return it.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPciConfiguration`
    /// * The read is sound i.e. it has no memory-safety related side-effects.
    unsafe fn read_pci_config_u8(
        &mut self,
        id: AcpiPciId,
        register: usize,
    ) -> Result<u8, AcpiError>;

    /// Read a [`u16`] from the configuration space of the given PCI ID and return it.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPciConfiguration`
    /// * The read is sound i.e. it has no memory-safety related side-effects.
    unsafe fn read_pci_config_u16(
        &mut self,
        id: AcpiPciId,
        register: usize,
    ) -> Result<u16, AcpiError>;

    /// Read a [`u32`] from the configuration space of the given PCI ID and return it.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPciConfiguration`
    /// * The read is sound i.e. it has no memory-safety related side-effects.
    unsafe fn read_pci_config_u32(
        &mut self,
        id: AcpiPciId,
        register: usize,
    ) -> Result<u32, AcpiError>;

    /// Read a [`u64`] from the configuration space of the given PCI ID and return it.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsReadPciConfiguration`
    /// * The read is sound i.e. it has no memory-safety related side-effects.
    unsafe fn read_pci_config_u64(
        &mut self,
        id: AcpiPciId,
        register: usize,
    ) -> Result<u64, AcpiError>;

    /// Write a [`u8`] to the configuration space of the given PCI ID.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsWritePciConfiguration`
    /// * The write is sound i.e. it has no memory-safety related side-effects.
    unsafe fn write_pci_config_u8(
        &mut self,
        id: AcpiPciId,
        register: usize,
        value: u8,
    ) -> Result<(), AcpiError>;

    /// Write a [`u16`] to the configuration space of the given PCI ID.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsRWriteciConfiguration`
    /// * The write is sound i.e. it has no memory-safety related side-effects.
    unsafe fn write_pci_config_u16(
        &mut self,
        id: AcpiPciId,
        register: usize,
        value: u16,
    ) -> Result<(), AcpiError>;

    /// Write a [`u32`] to the configuration space of the given PCI ID.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsRWriteciConfiguration`
    /// * The write is sound i.e. it has no memory-safety related side-effects.
    unsafe fn write_pci_config_u32(
        &mut self,
        id: AcpiPciId,
        register: usize,
        value: u32,
    ) -> Result<(), AcpiError>;

    /// Write a [`u64`] to the configuration space of the given PCI ID.
    /// `register` is the offset of the value to read in bytes.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsRWriteciConfiguration`
    /// * The write is sound i.e. it has no memory-safety related side-effects.
    unsafe fn write_pci_config_u64(
        &mut self,
        id: AcpiPciId,
        register: usize,
        value: u64,
    ) -> Result<(), AcpiError>;

    /// Called when the AML `Fatal` opcode is encountered. The OS can return from this method, or kill the thread executing the AML.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsSignal`
    unsafe fn signal_fatal(
        &mut self,
        fatal_type: u32,
        code: u32,
        argument: u32,
    ) -> Result<(), AcpiError>;

    /// Called when the AML `Breakpoint` opcode is encountered.
    ///
    /// # Safety
    /// * This method is only called from `AcpiOsSignal`
    unsafe fn signal_breakpoint(&mut self, message: &str) -> Result<(), AcpiError>;

    // TODO: Verify the info in the docs for these cache methods

    /// Creates a cache for ACPICA to store objects in to avoid lots of small heap allocations.
    ///
    /// This method is only present in the trait if the `builtin_cache` feature is not set.
    /// Otherwise, ACPICA's builtin implementation is used.
    ///
    /// The cache stores up to `max_depth` objects of size `object_size`.
    /// The OS is responsible for allocating and de-allocating objects within the cache.
    ///
    /// The OS returns a type-erased pointer which can safely be passed via FFI,
    /// but the pointer may point to any type.
    ///
    /// # Safety
    /// * This method is only called from `AcpiCreateCache`.
    ///
    /// [`Vec`]: alloc::vec::Vec
    /// [`BitVec`]: bitvec::vec::BitVec
    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn create_cache(
        &mut self,
        cache_name: &str,
        object_size: u16,
        max_depth: u16,
    ) -> Result<*mut c_void, AcpiError>;

    /// Deletes a cache which was previously created by [`create_cache`].
    ///
    /// This method is only present in the trait if the `builtin_cache` feature is not set.
    ///
    /// The OS is responsible for deallocating the backing memory of the cache.
    ///
    /// # Safety
    /// * This method is only called from `AcpiDeleteCache`
    /// * `cache` is a pointer which was previously returned from [`create_cache`]
    /// * After this method is called, other cache methods will not be called for this cache
    ///
    /// [`create_cache`]: AcpiHandler::create_cache
    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn delete_cache(&mut self, cache: *mut c_void) -> Result<(), AcpiAllocationError>;

    /// Removes all items from a cache.
    ///
    /// This method is only present in the trait if the `builtin_cache` feature is not set
    ///
    /// This method should mark all slots in the cache as empty, but not deallocate the backing memory
    ///
    /// # Safety
    /// * This method is only called from `AcpiPurgeCache`
    /// * `cache` is a pointer which was previously returned from [`create_cache`]
    ///
    /// [`create_cache`]: AcpiHandler::create_cache
    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn purge_cache(&mut self, cache: *mut c_void);

    /// Allocates an object inside a cache.
    ///
    /// This method is only present in the trait if the `builtin_cache` feature is not set.
    ///
    /// This method should return a pointer to a free slot in the cache, or `None` if all slots are full.
    ///
    /// # Safety
    /// * This method is only called from `AcpiPurgeCache`.
    /// * `cache` is a pointer which was previously returned from [`create_cache`].
    ///
    /// # Implementation safety
    /// * The returned pointer must be free for writes for the object size passed to [`create_cache`]
    ///     - i.e. it must not be being used by rust code, and it must not have been returned from this method before,
    ///     unless it has been explicitly freed using [`release_object`] or [`purge_cache`].
    ///
    /// [`create_cache`]: AcpiHandler::create_cache
    /// [`release_object`]: AcpiHandler::release_object
    /// [`purge_cache`]: AcpiHandler::purge_cache
    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn acquire_object(&mut self, cache: *mut c_void) -> Option<*mut u8>;

    /// Marks an object as free in a cache.
    ///
    /// This method is only present in the trait if the `builtin_cache` feature is not set.
    ///
    /// This method should mark the given object within the cache as free - i.e. allow it to be allocated again by [`acquire_object`].
    ///
    /// # Safety
    /// * This method is only called from `AcpiReleaseObject`.
    /// * `cache` is a pointer which was previously returned from [`create_cache`].
    /// * `object` is a pointer which was previously returned from [`acquire_object`].
    ///
    /// [`acquire_object`]: AcpiHandler::acquire_object
    /// [`create_cache`]: AcpiHandler::create_cache
    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn release_object(&mut self, cache: *mut c_void, object: *mut u8);

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn create_lock(&mut self) -> Result<*mut c_void, AcpiError>;

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn delete_lock(&mut self, lock: *mut c_void);

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn acquire_lock(&mut self, handle: *mut c_void) -> AcpiCpuFlags;

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn release_lock(&mut self, handle: *mut c_void, flags: AcpiCpuFlags);

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn create_semaphore(
        &mut self,
        max_units: u32,
        initial_units: u32,
    ) -> Result<*mut c_void, AcpiError>;

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn delete_semaphore(&mut self, handle: *mut c_void) -> Result<(), AcpiError>;

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn wait_semaphore(
        &mut self,
        handle: *mut c_void,
        units: u32,
        timeout: u16,
    ) -> Result<(), AcpiError>;

    #[allow(missing_docs)] // TODO: docs
    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn signal_semaphore(&mut self, handle: *mut c_void, units: u32)
        -> Result<(), AcpiError>;
}
