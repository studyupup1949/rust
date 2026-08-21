//! AARCH64 helpers to access system registers, control cache, barrier

#![no_std]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(extended_key_value_attributes)]
#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(dead_code)]
#![deny(missing_docs)]

/// AARCH64 cache helpers
pub mod cache;
#[macro_use]
#[rustfmt::skip]
mod macro_def;
/// AARCH64 mmu helpers
pub mod mmu;
/// AARCH64 system operations helpers
#[rustfmt::skip]
pub mod sysop;
/// AARCH64 system registers helpers
#[rustfmt::skip]
pub mod sysreg;

/// Secure Monitor Call causes an exception to EL3.
#[cfg(target_arch = "aarch64")]
pub fn smc() {
    unsafe {
        asm!("smc #0");
    };
}

/// Exception Return using the ELR and SPSR for the current Exception level
#[cfg(target_arch = "aarch64")]
pub fn eret() {
    unsafe {
        asm!("eret");
    };
}

///Check if an EL is implemented from AA64PFR0 register fields.
///'el' argument must be one of 1, 2 or 3.
#[cfg(target_arch = "aarch64")]
pub fn el_implemented(el: u8) -> bool {
    let shift = match el {
        0 => return true,
        1 => 4,
        2 => 8,
        3 => 12,
        _ => panic!("Wrong EL"),
    };
    ((sysreg::read_id_aa64dfr0_el1() >> shift) & 0xF) != 0
}

// Previously defined accesor functions with incomplete register names

/// Read current exception level
#[cfg(target_arch = "aarch64")]
pub fn read_current_el() -> u64 {
    sysreg::read_CurrentEl()
}

/// Data Synchronization Barrier
/// No instruction in program order after this instruction executes until this instruction completes.
#[cfg(target_arch = "aarch64")]
pub fn dsb() {
    sysop::dsb_sy()
}

/// Read Main ID Register
#[cfg(target_arch = "aarch64")]
pub fn read_midr() -> u64 {
    sysreg::read_midr_el1()
}

/// Read Multiprocessor Affinity Register
#[cfg(target_arch = "aarch64")]
pub fn read_mpidr() -> u64 {
    sysreg::read_mpidr_el1()
}

/// Read Secure Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn read_scr() -> u64 {
    sysreg::read_scr_el3()
}

/// Write Secure Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn write_scr(v: u64) {
    sysreg::write_scr_el3(v)
}

/// Read Hypervisor Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn read_hcr() -> u64 {
    sysreg::read_hcr_el2()
}

/// Write Hypervisor Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn write_hcr(v: u64) {
    sysreg::write_hcr_el2(v)
}

///Read Architectural Feature Access Control Register
#[cfg(target_arch = "aarch64")]
pub fn read_cpacr() -> u64 {
    sysreg::read_cpacr_el1()
}

///Write Architectural Feature Access Control Register
#[cfg(target_arch = "aarch64")]
pub fn write_cpacr(v: u64) {
    sysreg::write_cpacr_el1(v)
}
