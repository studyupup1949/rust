#[cfg(not(feature = "builtin_lock"))]
use crate::types::AcpiCpuFlags;
#[cfg(not(all(
    feature = "builtin_cache",
    feature = "builtin_lock",
    feature = "builtin_semaphore"
)))]
use core::ffi::c_void;

use alloc::boxed::Box;

use crate::{handler::AcpiHandler, status::AcpiError, types::tables::AcpiTableHeader};

#[allow(clippy::type_complexity)]
pub struct DummyHandler<'a> {
    pub fn_initialize: Box<dyn Fn() -> Result<(), AcpiError>>,
    pub fn_terminate: Box<dyn Fn() -> Result<(), AcpiError>>,

    pub fn_predefined_override: Box<
        dyn Fn(
            &crate::types::AcpiPredefinedNames,
        ) -> Result<Option<alloc::string::String>, crate::status::AcpiError>,
    >,

    pub fn_table_override: Box<
        dyn Fn(&AcpiTableHeader) -> Result<Option<AcpiTableHeader<'a>>, crate::status::AcpiError>,
    >,

    pub fn_physical_table_override: Box<
        dyn Fn(
            &AcpiTableHeader,
        ) -> Result<
            Option<(crate::types::AcpiPhysicalAddress, u32)>,
            crate::status::AcpiError,
        >,
    >,

    pub fn_get_root_pointer: Box<dyn Fn() -> crate::types::AcpiPhysicalAddress>,
    pub fn_map_memory: Box<
        dyn Fn(
            crate::types::AcpiPhysicalAddress,
            usize,
        ) -> Result<*mut u8, crate::types::AcpiMappingError>,
    >,

    pub fn_unmap_memory: Box<dyn Fn(*mut u8, usize)>,

    pub fn_get_physical_address: Box<
        dyn Fn(
            *mut u8,
        )
            -> Result<Option<crate::types::AcpiPhysicalAddress>, crate::status::AcpiError>,
    >,

    pub fn_install_interrupt_handler: Box<
        dyn Fn(u32, crate::types::AcpiInterruptCallback) -> Result<(), crate::status::AcpiError>,
    >,

    pub fn_remove_interrupt_handler: Box<
        dyn Fn(u32, crate::types::AcpiInterruptCallbackTag) -> Result<(), crate::status::AcpiError>,
    >,

    pub fn_get_thread_id: Box<dyn Fn() -> u64>,

    pub fn_execute:
        Box<dyn Fn(crate::types::AcpiThreadCallback) -> Result<(), crate::status::AcpiError>>,

    pub fn_wait_for_events: Box<dyn Fn()>,

    pub fn_printf: Box<dyn Fn(core::fmt::Arguments)>,

    pub fn_sleep: Box<dyn Fn(usize)>,

    pub fn_stall: Box<dyn Fn(usize)>,

    pub fn_read_port_u8: Box<dyn Fn(crate::types::AcpiIoAddress) -> Result<u8, AcpiError>>,

    pub fn_read_port_u16: Box<dyn Fn(crate::types::AcpiIoAddress) -> Result<u16, AcpiError>>,

    pub fn_read_port_u32: Box<dyn Fn(crate::types::AcpiIoAddress) -> Result<u32, AcpiError>>,

    pub fn_write_port_u8: Box<dyn Fn(crate::types::AcpiIoAddress, u8) -> Result<(), AcpiError>>,

    pub fn_write_port_u16: Box<dyn Fn(crate::types::AcpiIoAddress, u16) -> Result<(), AcpiError>>,

    pub fn_write_port_u32: Box<dyn Fn(crate::types::AcpiIoAddress, u32) -> Result<(), AcpiError>>,

    pub fn_enter_sleep: Box<dyn Fn(u8, u32, u32) -> Result<(), AcpiError>>,

    pub fn_get_timer: Box<dyn Fn() -> u64>,

    pub fn_read_physical_u8:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress) -> Result<u8, AcpiError>>,

    pub fn_read_physical_u16:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress) -> Result<u16, AcpiError>>,

    pub fn_read_physical_u32:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress) -> Result<u32, AcpiError>>,

    pub fn_read_physical_u64:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress) -> Result<u64, AcpiError>>,

    pub fn_write_physical_u8:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress, u8) -> Result<(), AcpiError>>,

    pub fn_write_physical_u16:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress, u16) -> Result<(), AcpiError>>,

    pub fn_write_physical_u32:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress, u32) -> Result<(), AcpiError>>,

    pub fn_write_physical_u64:
        Box<dyn Fn(crate::types::AcpiPhysicalAddress, u64) -> Result<(), AcpiError>>,

    pub fn_readable: Box<dyn Fn(*mut core::ffi::c_void, usize) -> bool>,

    pub fn_writable: Box<dyn Fn(*mut core::ffi::c_void, usize) -> bool>,

    pub fn_read_pci_config_u8: Box<dyn Fn(crate::types::AcpiPciId, usize) -> Result<u8, AcpiError>>,

    pub fn_read_pci_config_u16:
        Box<dyn Fn(crate::types::AcpiPciId, usize) -> Result<u16, AcpiError>>,

    pub fn_read_pci_config_u32:
        Box<dyn Fn(crate::types::AcpiPciId, usize) -> Result<u32, AcpiError>>,

    pub fn_read_pci_config_u64:
        Box<dyn Fn(crate::types::AcpiPciId, usize) -> Result<u64, AcpiError>>,

    pub fn_write_pci_config_u8:
        Box<dyn Fn(crate::types::AcpiPciId, usize, u8) -> Result<(), AcpiError>>,

    pub fn_write_pci_config_u16:
        Box<dyn Fn(crate::types::AcpiPciId, usize, u16) -> Result<(), AcpiError>>,

    pub fn_write_pci_config_u32:
        Box<dyn Fn(crate::types::AcpiPciId, usize, u32) -> Result<(), AcpiError>>,

    pub fn_write_pci_config_u64:
        Box<dyn Fn(crate::types::AcpiPciId, usize, u64) -> Result<(), AcpiError>>,

    pub fn_signal_fatal: Box<dyn Fn(u32, u32, u32) -> Result<(), AcpiError>>,

    pub fn_signal_breakpoint: Box<dyn Fn(&str) -> Result<(), AcpiError>>,

    #[cfg(not(feature = "builtin_cache"))]
    pub fn_create_cache: Box<dyn Fn(&str, u16, u16) -> Result<*mut c_void, AcpiError>>,

    #[cfg(not(feature = "builtin_cache"))]
    pub fn_delete_cache: Box<dyn Fn(*mut c_void) -> Result<(), AcpiAllocationError>>,

    #[cfg(not(feature = "builtin_cache"))]
    pub fn_purge_cache: Box<dyn Fn(*mut c_void)>,

    #[cfg(not(feature = "builtin_cache"))]
    pub fn_acquire_object: Box<dyn Fn(*mut c_void) -> Option<*mut u8>>,

    #[cfg(not(feature = "builtin_cache"))]
    pub fn_release_object: Box<dyn Fn(*mut c_void, *mut u8)>,

    #[cfg(not(feature = "builtin_lock"))]
    pub fn_create_lock: Box<dyn Fn() -> Result<*mut c_void, AcpiError>>,

    #[cfg(not(feature = "builtin_lock"))]
    pub fn_delete_lock: Box<dyn Fn(*mut c_void)>,

    #[cfg(not(feature = "builtin_lock"))]
    pub fn_acquire_lock: Box<dyn Fn(*mut c_void) -> AcpiCpuFlags>,

    #[cfg(not(feature = "builtin_lock"))]
    pub fn_release_lock: Box<dyn Fn(*mut c_void, AcpiCpuFlags)>,

    #[cfg(not(feature = "builtin_semaphore"))]
    pub fn_create_semaphore: Box<dyn Fn(u32, u32) -> Result<*mut c_void, AcpiError>>,

    #[cfg(not(feature = "builtin_semaphore"))]
    pub fn_delete_semaphore: Box<dyn Fn(*mut c_void) -> Result<(), AcpiError>>,

    #[cfg(not(feature = "builtin_semaphore"))]
    pub fn_wait_semaphore: Box<dyn Fn(*mut c_void, u32, u16) -> Result<(), AcpiError>>,

    #[cfg(not(feature = "builtin_semaphore"))]
    pub fn_signal_semaphore: Box<dyn Fn(*mut c_void, u32) -> Result<(), AcpiError>>,
}

// SAFETY: This struct is only used in tests, which are carried out in a no_std environment not linked to any OS
// Therefore all test code is single threaded and so this impl is sound
unsafe impl Send for DummyHandler<'static> {}

impl<'a> DummyHandler<'a> {
    pub(crate) fn new() -> Self {
        fn dummy_0_arg<T>() -> T {
            panic!("Dummy function on test struct called")
        }
        fn dummy_1_arg<T, U>(_: U) -> T {
            panic!("Dummy function on test struct called")
        }
        fn dummy_2_arg<T, U, V>(_: U, _: V) -> T {
            panic!("Dummy function on test struct called")
        }
        fn dummy_3_arg<T, U, V, W>(_: U, _: V, _: W) -> T {
            panic!("Dummy function on test struct called")
        }

        Self {
            fn_get_root_pointer: Box::new(dummy_0_arg),
            fn_map_memory: Box::new(dummy_2_arg),
            fn_unmap_memory: Box::new(dummy_2_arg),
            fn_get_physical_address: Box::new(dummy_1_arg),
            fn_install_interrupt_handler: Box::new(dummy_2_arg),
            fn_remove_interrupt_handler: Box::new(dummy_2_arg),
            fn_get_thread_id: Box::new(dummy_0_arg),
            fn_execute: Box::new(dummy_1_arg),
            fn_wait_for_events: Box::new(dummy_0_arg),
            fn_printf: Box::new(|_| panic!("Dummy function on test struct called")),
            fn_initialize: Box::new(dummy_0_arg),
            fn_terminate: Box::new(dummy_0_arg),
            fn_predefined_override: Box::new(|_| panic!("Dummy function on test struct called")),
            fn_table_override: Box::new(|_| panic!("Dummy function on test struct called")),
            fn_physical_table_override: Box::new(|_| {
                panic!("Dummy function on test struct called")
            }),

            fn_sleep: Box::new(dummy_1_arg),
            fn_stall: Box::new(dummy_1_arg),
            fn_read_port_u8: Box::new(dummy_1_arg),
            fn_read_port_u16: Box::new(dummy_1_arg),
            fn_read_port_u32: Box::new(dummy_1_arg),
            fn_write_port_u8: Box::new(dummy_2_arg),
            fn_write_port_u16: Box::new(dummy_2_arg),
            fn_write_port_u32: Box::new(dummy_2_arg),

            fn_enter_sleep: Box::new(dummy_3_arg),
            fn_get_timer: Box::new(dummy_0_arg),

            fn_read_physical_u8: Box::new(dummy_1_arg),
            fn_read_physical_u16: Box::new(dummy_1_arg),
            fn_read_physical_u32: Box::new(dummy_1_arg),
            fn_read_physical_u64: Box::new(dummy_1_arg),
            fn_write_physical_u8: Box::new(dummy_2_arg),
            fn_write_physical_u16: Box::new(dummy_2_arg),
            fn_write_physical_u32: Box::new(dummy_2_arg),
            fn_write_physical_u64: Box::new(dummy_2_arg),
            fn_readable: Box::new(dummy_2_arg),
            fn_writable: Box::new(dummy_2_arg),

            fn_read_pci_config_u8: Box::new(dummy_2_arg),
            fn_read_pci_config_u16: Box::new(dummy_2_arg),
            fn_read_pci_config_u32: Box::new(dummy_2_arg),
            fn_read_pci_config_u64: Box::new(dummy_2_arg),
            fn_write_pci_config_u8: Box::new(dummy_3_arg),
            fn_write_pci_config_u16: Box::new(dummy_3_arg),
            fn_write_pci_config_u32: Box::new(dummy_3_arg),
            fn_write_pci_config_u64: Box::new(dummy_3_arg),

            fn_signal_breakpoint: Box::new(|_| panic!("Dummy function on test struct called")),
            fn_signal_fatal: Box::new(dummy_3_arg),

            #[cfg(not(feature = "builtin_cache"))]
            fn_create_cache: Box::new(dummy_3_arg),
            #[cfg(not(feature = "builtin_cache"))]
            fn_delete_cache: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_cache"))]
            fn_purge_cache: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_cache"))]
            fn_acquire_object: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_cache"))]
            fn_release_object: Box::new(dummy_2_arg),

            #[cfg(not(feature = "builtin_lock"))]
            fn_create_lock: Box::new(dummy_0_arg),
            #[cfg(not(feature = "builtin_lock"))]
            fn_delete_lock: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_lock"))]
            fn_acquire_lock: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_lock"))]
            fn_release_lock: Box::new(dummy_2_arg),

            #[cfg(not(feature = "builtin_semaphore"))]
            fn_create_semaphore: Box::new(dummy_2_arg),
            #[cfg(not(feature = "builtin_semaphore"))]
            fn_delete_semaphore: Box::new(dummy_1_arg),
            #[cfg(not(feature = "builtin_semaphore"))]
            fn_wait_semaphore: Box::new(dummy_3_arg),
            #[cfg(not(feature = "builtin_semaphore"))]
            fn_signal_semaphore: Box::new(dummy_2_arg),
        }
    }
}

// SAFETY:
// Each method in this implementation is the user of the test struct's responsibility
unsafe impl<'a> AcpiHandler for DummyHandler<'a> {
    fn get_root_pointer(&mut self) -> crate::types::AcpiPhysicalAddress {
        (self.fn_get_root_pointer)()
    }

    unsafe fn map_memory(
        &mut self,
        physical_address: crate::types::AcpiPhysicalAddress,
        length: usize,
    ) -> Result<*mut u8, crate::types::AcpiMappingError> {
        (self.fn_map_memory)(physical_address, length)
    }

    unsafe fn unmap_memory(&mut self, address: *mut u8, length: usize) {
        (self.fn_unmap_memory)(address, length);
    }

    fn get_physical_address(
        &mut self,
        logical_address: *mut u8,
    ) -> Result<Option<crate::types::AcpiPhysicalAddress>, crate::status::AcpiError> {
        (self.fn_get_physical_address)(logical_address)
    }

    unsafe fn install_interrupt_handler(
        &mut self,
        interrupt_number: u32,
        callback: crate::types::AcpiInterruptCallback,
    ) -> Result<(), crate::status::AcpiError> {
        (self.fn_install_interrupt_handler)(interrupt_number, callback)
    }

    unsafe fn remove_interrupt_handler(
        &mut self,
        interrupt_number: u32,
        callback: crate::types::AcpiInterruptCallbackTag,
    ) -> Result<(), crate::status::AcpiError> {
        (self.fn_remove_interrupt_handler)(interrupt_number, callback)
    }

    fn get_thread_id(&mut self) -> u64 {
        (self.fn_get_thread_id)()
    }

    unsafe fn execute(
        &mut self,
        // callback_type: AcpiExecuteType,
        callback: crate::types::AcpiThreadCallback,
    ) -> Result<(), crate::status::AcpiError> {
        (self.fn_execute)(callback)
    }

    unsafe fn wait_for_events(&mut self) {
        (self.fn_wait_for_events)();
    }

    fn printf(&mut self, message: core::fmt::Arguments) {
        (self.fn_printf)(message);
    }

    unsafe fn sleep(&mut self, millis: usize) {
        (self.fn_sleep)(millis);
    }

    unsafe fn stall(&mut self, micros: usize) {
        (self.fn_stall)(micros);
    }

    unsafe fn read_port_u8(
        &mut self,
        address: crate::types::AcpiIoAddress,
    ) -> Result<u8, AcpiError> {
        (self.fn_read_port_u8)(address)
    }

    unsafe fn read_port_u16(
        &mut self,
        address: crate::types::AcpiIoAddress,
    ) -> Result<u16, AcpiError> {
        (self.fn_read_port_u16)(address)
    }

    unsafe fn read_port_u32(
        &mut self,
        address: crate::types::AcpiIoAddress,
    ) -> Result<u32, AcpiError> {
        (self.fn_read_port_u32)(address)
    }

    unsafe fn write_port_u8(
        &mut self,
        address: crate::types::AcpiIoAddress,
        value: u8,
    ) -> Result<(), AcpiError> {
        (self.fn_write_port_u8)(address, value)
    }

    unsafe fn write_port_u16(
        &mut self,
        address: crate::types::AcpiIoAddress,
        value: u16,
    ) -> Result<(), AcpiError> {
        (self.fn_write_port_u16)(address, value)
    }

    unsafe fn write_port_u32(
        &mut self,
        address: crate::types::AcpiIoAddress,
        value: u32,
    ) -> Result<(), AcpiError> {
        (self.fn_write_port_u32)(address, value)
    }

    unsafe fn initialize(&mut self) -> Result<(), crate::status::AcpiError> {
        (self.fn_initialize)()
    }

    unsafe fn terminate(&mut self) -> Result<(), crate::status::AcpiError> {
        (self.fn_terminate)()
    }

    unsafe fn predefined_override(
        &mut self,
        predefined_object: &crate::types::AcpiPredefinedNames,
    ) -> Result<Option<alloc::string::String>, crate::status::AcpiError> {
        (self.fn_predefined_override)(predefined_object)
    }

    unsafe fn table_override(
        &mut self,
        table: &AcpiTableHeader,
    ) -> Result<Option<AcpiTableHeader>, crate::status::AcpiError> {
        (self.fn_table_override)(table)
    }

    unsafe fn physical_table_override(
        &mut self,
        table: &AcpiTableHeader,
    ) -> Result<Option<(crate::types::AcpiPhysicalAddress, u32)>, crate::status::AcpiError> {
        (self.fn_physical_table_override)(table)
    }

    unsafe fn enter_sleep(&mut self, state: u8, reg_a: u32, reg_b: u32) -> Result<(), AcpiError> {
        (self.fn_enter_sleep)(state, reg_a, reg_b)
    }

    unsafe fn get_timer(&mut self) -> u64 {
        (self.fn_get_timer)()
    }

    unsafe fn read_physical_u8(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
    ) -> Result<u8, AcpiError> {
        (self.fn_read_physical_u8)(address)
    }

    unsafe fn read_physical_u16(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
    ) -> Result<u16, AcpiError> {
        (self.fn_read_physical_u16)(address)
    }

    unsafe fn read_physical_u32(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
    ) -> Result<u32, AcpiError> {
        (self.fn_read_physical_u32)(address)
    }

    unsafe fn read_physical_u64(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
    ) -> Result<u64, AcpiError> {
        (self.fn_read_physical_u64)(address)
    }

    unsafe fn write_physical_u8(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
        value: u8,
    ) -> Result<(), AcpiError> {
        (self.fn_write_physical_u8)(address, value)
    }

    unsafe fn write_physical_u16(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
        value: u16,
    ) -> Result<(), AcpiError> {
        (self.fn_write_physical_u16)(address, value)
    }

    unsafe fn write_physical_u32(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
        value: u32,
    ) -> Result<(), AcpiError> {
        (self.fn_write_physical_u32)(address, value)
    }

    unsafe fn write_physical_u64(
        &mut self,
        address: crate::types::AcpiPhysicalAddress,
        value: u64,
    ) -> Result<(), AcpiError> {
        (self.fn_write_physical_u64)(address, value)
    }

    unsafe fn readable(&mut self, pointer: *mut core::ffi::c_void, length: usize) -> bool {
        (self.fn_readable)(pointer, length)
    }

    unsafe fn writable(&mut self, pointer: *mut core::ffi::c_void, length: usize) -> bool {
        (self.fn_writable)(pointer, length)
    }

    unsafe fn read_pci_config_u8(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
    ) -> Result<u8, AcpiError> {
        (self.fn_read_pci_config_u8)(id, register)
    }

    unsafe fn read_pci_config_u16(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
    ) -> Result<u16, AcpiError> {
        (self.fn_read_pci_config_u16)(id, register)
    }

    unsafe fn read_pci_config_u32(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
    ) -> Result<u32, AcpiError> {
        (self.fn_read_pci_config_u32)(id, register)
    }

    unsafe fn read_pci_config_u64(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
    ) -> Result<u64, AcpiError> {
        (self.fn_read_pci_config_u64)(id, register)
    }

    unsafe fn write_pci_config_u8(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
        value: u8,
    ) -> Result<(), AcpiError> {
        (self.fn_write_pci_config_u8)(id, register, value)
    }

    unsafe fn write_pci_config_u16(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
        value: u16,
    ) -> Result<(), AcpiError> {
        (self.fn_write_pci_config_u16)(id, register, value)
    }

    unsafe fn write_pci_config_u32(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
        value: u32,
    ) -> Result<(), AcpiError> {
        (self.fn_write_pci_config_u32)(id, register, value)
    }

    unsafe fn write_pci_config_u64(
        &mut self,
        id: crate::types::AcpiPciId,
        register: usize,
        value: u64,
    ) -> Result<(), AcpiError> {
        (self.fn_write_pci_config_u64)(id, register, value)
    }

    unsafe fn signal_fatal(
        &mut self,
        fatal_type: u32,
        code: u32,
        argument: u32,
    ) -> Result<(), AcpiError> {
        (self.fn_signal_fatal)(fatal_type, code, argument)
    }

    unsafe fn signal_breakpoint(&mut self, message: &str) -> Result<(), AcpiError> {
        (self.fn_signal_breakpoint)(message)
    }

    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn create_cache(
        &mut self,
        cache_name: &str,
        object_size: u16,
        max_depth: u16,
    ) -> Result<*mut c_void, AcpiError> {
        (self.fn_create_cache)(cache_name, object_size, max_depth)
    }

    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn delete_cache(&mut self, cache: *mut c_void) -> Result<(), AcpiAllocationError> {
        (self.fn_delete_cache)(cache)
    }

    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn purge_cache(&mut self, cache: *mut c_void) {
        (self.fn_purge_cache)(cache)
    }

    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn acquire_object(&mut self, cache: *mut c_void) -> Option<*mut u8> {
        (self.fn_acquire_object)(cache)
    }

    #[cfg(not(feature = "builtin_cache"))]
    unsafe fn release_object(&mut self, cache: *mut c_void, object: *mut u8) {
        (self.fn_release_object)(cache, object)
    }

    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn create_lock(&mut self) -> Result<*mut c_void, AcpiError> {
        (self.fn_create_lock)()
    }

    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn delete_lock(&mut self, lock: *mut c_void) {
        (self.fn_delete_lock)(lock)
    }

    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn acquire_lock(&mut self, handle: *mut c_void) -> AcpiCpuFlags {
        (self.fn_acquire_lock)(handle)
    }

    #[cfg(not(feature = "builtin_lock"))]
    unsafe fn release_lock(&mut self, handle: *mut c_void, flags: AcpiCpuFlags) {
        (self.fn_release_lock)(handle, flags)
    }

    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn create_semaphore(
        &mut self,
        max_units: u32,
        initial_units: u32,
    ) -> Result<*mut c_void, AcpiError> {
        (self.fn_create_semaphore)(max_units, initial_units)
    }

    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn delete_semaphore(&mut self, handle: *mut c_void) -> Result<(), AcpiError> {
        (self.fn_delete_semaphore)(handle)
    }

    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn wait_semaphore(
        &mut self,
        handle: *mut c_void,
        units: u32,
        timeout: u16,
    ) -> Result<(), AcpiError> {
        (self.fn_wait_semaphore)(handle, units, timeout)
    }

    #[cfg(not(feature = "builtin_semaphore"))]
    unsafe fn signal_semaphore(
        &mut self,
        handle: *mut c_void,
        units: u32,
    ) -> Result<(), AcpiError> {
        (self.fn_signal_semaphore)(handle, units)
    }
}
