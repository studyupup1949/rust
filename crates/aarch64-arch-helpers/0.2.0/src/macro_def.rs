macro_rules! sysreg_read_func {
    ( $name: ident, $reg_name: tt, $desc: tt ) => {
        #[doc=concat!("Read value from register ", $reg_name)]
        #[cfg(target_arch = "aarch64")]
        pub fn $name() -> u64 {
            let mut v: u64;
            unsafe {
                llvm_asm!(concat!("mrs $0, ", $reg_name)
                    : "=r"(v)
                );
            }
            v
        }
    };
}

macro_rules! sysreg_write_func {
    ( $name: ident, $reg_name: tt, $desc: tt) => {
        #[doc=concat!("Write value to register ", $reg_name)]
        #[cfg(target_arch = "aarch64")]
        pub fn $name(v: u64) {
            unsafe {
                llvm_asm!(concat!("msr ", $reg_name, ", $0")
                    :
                    : "r"(v));
            }
        }
    }
}

macro_rules! sysreg_rw_func {
    ( $read_name: ident, $write_name: ident, $reg_name: tt, $desc: tt) => {
        sysreg_read_func!($read_name, $reg_name, $desc);
        sysreg_write_func!($write_name, $reg_name, $desc);
    }
}

/// Define function for simple system instruction
macro_rules! sysop_func {
    ( $fname: ident ) => {
        #[doc=concat!("Call ", stringify!($fname))]
        #[cfg(target_arch = "aarch64")]
        pub fn $fname() {
            unsafe {
                asm!(stringify!($fname));
            }
        }
    };
}

/// Define function for system instruction with type specifier
macro_rules! sysop_type_func {
    ( $fname: ident, $op: tt, $op_type: tt, $desc: tt) => {
        #[doc = concat!("Call ", $op, " with type ", $op_type, "

        ", $desc)]
        #[cfg(target_arch = "aarch64")]
        pub fn $fname() {
            unsafe {
                asm!(concat!($op, " ", $op_type));
            }
        }
    }
}

/// Define function for system instruction with register parameter
macro_rules! sysop_type_param_func {
    ( $fname: ident, $op: tt, $op_type: tt, $desc: tt) => {
        #[doc=concat!("Call ", $op, " with type ", $op_type, "

        ", $desc)]
        #[cfg(target_arch = "aarch64")]
        pub fn $fname(v: u64) {
            unsafe {
                llvm_asm!(concat!($op, " ", $op_type, ", $0")
                    :
                    : "r"(v));
            }
        }
    };
}

/// Write constant in range of 0..16 to system register
macro_rules! sysreg_write_const {
    ( $name: ident, $reg_name: tt) => {
        macro_rules! $name {
            ($val: expr) => {
                unsafe {
                    llvm_asm!(concat!("msr ", $reg_name, ", $0")
                        :
                        : "r"($val));
                }
            }
        }
    }
}
