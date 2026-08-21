pub mod cli;
pub mod error;

use cli::Type;

use ::windows::Win32::{
    Foundation::{GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE},
    Storage::FileSystem::{
        DELETE,
        READ_CONTROL,
        STANDARD_RIGHTS_ALL,
        STANDARD_RIGHTS_EXECUTE,
        STANDARD_RIGHTS_READ,
        STANDARD_RIGHTS_REQUIRED,
        STANDARD_RIGHTS_WRITE,
        SYNCHRONIZE,
        WRITE_DAC,
        WRITE_OWNER,
    },
    System::SystemServices::ACCESS_SYSTEM_SECURITY,
};

#[macro_export]
macro_rules! print_attribute {
    ($value:expr, $attribute:expr, $attribute_value:expr) => {{
        let attribute_value = u32::try_from($attribute_value).expect("value must fit in a u32");
        if $value & attribute_value == attribute_value {
            println!(
                "  {:>35} (0x{:08x})",
                stringify!($attribute),
                $attribute_value
            );
        }
    }};
}

pub fn parse_attributes(value: u32, typ: Option<Type>) {
    println!("Access mask: 0x{value:08x} ({value})\n");
    println!("Generic rights:");
    print_attribute!(value, ACCESS_SYSTEM_SECURITY, ACCESS_SYSTEM_SECURITY);

    print_attribute!(value, DELETE, DELETE.0);

    print_attribute!(value, GENERIC_ALL, GENERIC_ALL.0);
    print_attribute!(value, GENERIC_EXECUTE, GENERIC_EXECUTE.0);
    print_attribute!(value, GENERIC_READ, GENERIC_READ.0);
    print_attribute!(value, GENERIC_WRITE, GENERIC_WRITE.0);

    print_attribute!(value, READ_CONTROL, READ_CONTROL.0);

    print_attribute!(value, STANDARD_RIGHTS_ALL, STANDARD_RIGHTS_ALL.0);
    print_attribute!(value, STANDARD_RIGHTS_EXECUTE, STANDARD_RIGHTS_EXECUTE.0);
    print_attribute!(value, STANDARD_RIGHTS_READ, STANDARD_RIGHTS_READ.0);
    print_attribute!(value, STANDARD_RIGHTS_REQUIRED, STANDARD_RIGHTS_REQUIRED.0);
    print_attribute!(value, STANDARD_RIGHTS_WRITE, STANDARD_RIGHTS_WRITE.0);

    print_attribute!(value, SYNCHRONIZE, SYNCHRONIZE.0);

    print_attribute!(value, WRITE_DAC, WRITE_DAC.0);
    print_attribute!(value, WRITE_OWNER, WRITE_OWNER.0);

    if let Some(typ) = typ {
        typ.print_parsed_type(value);
    } else {
        println!("  (no object type) 0x{:08x}", value & 0xffff);
    }
}
