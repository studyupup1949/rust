#[export_name = "AcpiOsGetTableByName"]
extern "C" fn acpi_os_get_table_by_name(
    _signature: *mut i8,
    _instance: u32,
    _table: *mut *mut FfiAcpiTableHeader,
    _address: *mut FfiAcpiPhysicalAddress,
) -> AcpiStatus {
    todo!()
}

#[export_name = "AcpiOsGetTableByIndex"]
extern "C" fn acpi_os_get_table_by_index(
    _index: u32,
    _table: *mut *mut FfiAcpiTableHeader,
    _instance: *mut u32,
    _address: *mut FfiAcpiPhysicalAddress,
) -> AcpiStatus {
    todo!()
}

#[export_name = "AcpiOsGetTableByAddress"]
extern "C" fn acpi_os_get_table_by_address(
    _address: FfiAcpiPhysicalAddress,
    _table: *mut *mut FfiAcpiTableHeader,
) -> AcpiStatus {
    todo!()
}