use crate::sysop::{dsb_sy, isb};
use crate::sysreg::{read_sctlr_el1, read_sctlr_el3, write_sctlr_el1, write_sctlr_el3};

const SCTLR_M_BIT: u64 = 1 << 0;
const SCTLR_C_BIT: u64 = 1 << 2;
const SCTLR_I_BIT: u64 = 1 << 12;

/// Disable MMU and Data Cache (EL1)
#[cfg(target_arch = "aarch64")]
pub fn disable_mmu_el1() {
    let mut v = read_sctlr_el1();
    v &= !(SCTLR_M_BIT | SCTLR_C_BIT);
    write_sctlr_el1(v);
    isb();
    dsb_sy();
}

/// Disable instruction cache (EL1)
#[cfg(target_arch = "aarch64")]
pub fn disable_mmu_icache_el1() {
    let mut v = read_sctlr_el1();
    v &= !(SCTLR_M_BIT | SCTLR_C_BIT | SCTLR_I_BIT);
    write_sctlr_el1(v);
    isb();
    dsb_sy();
}

/// Disable MMU and Data Cache (EL3)
#[cfg(target_arch = "aarch64")]
pub fn disable_mmu_el3() {
    let mut v = read_sctlr_el3();
    v &= !(SCTLR_M_BIT | SCTLR_C_BIT);
    write_sctlr_el3(v);
    isb();
    dsb_sy();
}

/// Disable instruction cache (EL3)
#[cfg(target_arch = "aarch64")]
pub fn disable_mmu_icache_el3() {
    let mut v = read_sctlr_el3();
    v &= !(SCTLR_M_BIT | SCTLR_C_BIT | SCTLR_I_BIT);
    write_sctlr_el3(v);
    isb();
    dsb_sy();
}
