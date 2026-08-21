//! AARCH64 helpers to access system registers, control cache, barrier

#![no_std]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(extended_key_value_attributes)]
#![allow(non_snake_case)]
#![allow(unused_macros)]
#![deny(missing_docs)]

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

sysop_type_func!(tlbi_alle1, "tlbi", "alle1", "Invalidate all stage 1 translations used at EL1");
sysop_type_func!(tlbi_alle1is, "tlbi", "alle1is", "Invalidate all stage 1 translations used at EL1, Inner Shareable");
sysop_type_func!(tlbi_alle2, "tlbi", "alle2", "Invalidate all stage 1 translations used at EL2");
sysop_type_func!(tlbi_alle2is, "tlbi", "alle2is", "Invalidate all stage 1 translations used at EL2, Inner Shareable");
#[cfg(not(feature = "errata_a57_813419"))]
sysop_type_func!(tlbi_alle3, "tlbi", "alle3", "Invalidate all stage 1 translations used at EL3");
#[cfg(not(feature = "errata_a57_813419"))]
sysop_type_func!(tlbi_alle3is, "tlbi", "alle3is", "Invalidate all stage 1 translations used at EL3, Inner Shareable");
sysop_type_func!(tlbi_vmalle1, "tlbi", "vmalle1", "Invalidate all stage 1 translations used at EL1 with the current VMID");
sysop_type_param_func!(tlbi_vaae1is, "tlbi", "vaae1is", "Invalidate all translations used at EL1 for the specified address and current VMID and for all ASID values, Inner Shareable");
sysop_type_param_func!(tlbi_vaale1is, "tlbi", "vaale1is", "Invalidate all entries from the last level of stage 1 translation table walk used at EL1 for the specified address and current VMID and for all ASID values, Inner Shareable");
sysop_type_param_func!(tlbi_vae2is, "tlbi", "vae2is", "Invalidate translation used at EL2 for the specified VA and ASID and the current VMID, Inner Shareable");
sysop_type_param_func!(tlbi_vale2is, "tlbi", "vale2is", "Invalidate all entries from the last level of stage 1 translation table walk used at EL2 with the supplied ASID and current VMID, Inner Shareable");
#[cfg(not(feature = "errata_a57_813419"))]
sysop_type_param_func!(tlbi_vae3is, "tlbi", "vae3is", "Invalidate translation used at EL3 for the specified VA and ASID and the current VMID, Inner Shareable");
#[cfg(not(feature = "errata_a57_813419"))]
sysop_type_param_func!(tlbi_vale3is, "tlbi", "vale3is", "Invalidate all entries from the last level of stage 1 translation table walk used at EL3 with the supplied ASID and current VMID, Inner Shareable");

// Cache maintenance accessor prototypes
sysop_type_param_func!(dc_isw, "dc", "isw", "Data cache invalidate by set/way");
sysop_type_param_func!(dc_cisw, "dc", "cisw", "Data cache clean and invalidate by set/way");
sysop_type_param_func!(dc_csw, "dc", "csw", "Data cache clean by set/way");
sysop_type_param_func!(dc_cvac, "dc", "cvac", "Data cache clean by VA to PoC");
sysop_type_param_func!(dc_ivac, "dc", "ivac", "Data cache invalidate by VA to PoC");
sysop_type_param_func!(dc_civac, "dc", "civac", "Data cache clean and invalidate by VA to PoC");
sysop_type_param_func!(dc_cvau, "dc", "cvau", "Data cache clean by VA to PoU");
sysop_type_param_func!(dc_zva, "dc", "zva", "Data cache zero by VA");

// Address translation accessor prototypes
sysop_type_param_func!(at_s12e1r, "at", "s12e1r", "Stages 1 and 2 Non-secure EL1 read");
sysop_type_param_func!(at_s12e1w, "at", "s12e1w", "Stages 1 and 2 Non-secure EL1 write");
sysop_type_param_func!(at_s12e0r, "at", "s12e0r", "Stages 1 and 2 Non-secure unprivileged read");
sysop_type_param_func!(at_s12e0w, "at", "s12e0w", "Stages 1 and 2 Non-secure unprivileged write");
sysop_type_param_func!(at_s1e2r, "at", "s1e2r", "Stage 1 Hyp mode read");

// Misc accessor prototype
#[macro_export]
sysreg_write_const!(write_daifclr, "daifclr");
#[macro_export]
sysreg_write_const!(write_daifset, "daifset");
sysreg_read_func!(read_par_el1, "par_el1", "Physical Address Register, EL1");
sysreg_read_func!(read_id_pfr1_el1, "id_pfr1_el1", "Processor Feature Register 1, EL1");
sysreg_read_func!(read_id_aa64pfr0_el1, "id_aa64pfr0_el1", "AArch64 Processor Feature Register 0");
sysreg_read_func!(read_id_aa64dfr0_el1, "id_aa64dfr0_el1", "AArch64 Debug Feature Register 0");
sysreg_read_func!(read_CurrentEl, "CurrentEl", "The current exception level");
sysreg_rw_func!(read_daif, write_daif, "daif", "Interrupt Mask Bits");
sysreg_rw_func!(read_spsr_el1, write_spsr_el1, "spsr_el1", "Saved Program Status Register (EL1)");
sysreg_rw_func!(read_spsr_el2, write_spsr_el2, "spsr_el2", "Saved Program Status Register (EL2)");
sysreg_rw_func!(read_spsr_el3, write_spsr_el3, "spsr_el3", "Saved Program Status Register (EL3)");
sysreg_rw_func!(read_elr_el1, write_elr_el1, "elr_el1", "Exception Link Register (EL1)");
sysreg_rw_func!(read_elr_el2, write_elr_el2, "elr_el2", "Exception Link Register (EL2)");
sysreg_rw_func!(read_elr_el3, write_elr_el3, "elr_el3", "Exception Link Register (EL3)");

sysop_func!(wfi);
sysop_func!(wfe);
sysop_func!(sev);

sysop_type_func!(dsb_sy, "dsb", "sy", "Full system DSB operation");
sysop_type_func!(dmb_sy, "dmb", "sy", "Full system DMB operation");
sysop_type_func!(dmb_st, "dmb", "st", "DMB operation that waits only for stores to complete");
sysop_type_func!(dmb_ld, "dmb", "ld", "DMB operation that waits only for loads to complete");
sysop_type_func!(dsb_ish, "dsb", "ish", "DSB operation only to the inner shareable domain");
sysop_type_func!(dsb_nsh, "dsb", "nsh", "DSB operation only out to the point of unification");
sysop_type_func!(dsb_ishst, "dsb", "ishst", "DSB operation that waits only for stores to complete, and only to the inner shareable domain");
sysop_type_func!(dmb_ish, "dmb", "ish", "DMB operation only to the inner shareable domain");
sysop_type_func!(dmb_ishst, "dmb", "ishst", "DMB operation that waits only for stores to complete, and only to the inner shareable domain");

sysop_func!(isb);

// System register accessor prototypes
sysreg_read_func!(read_midr_el1, "midr_el1", "Main ID Register");
sysreg_read_func!(read_mpidr_el1, "mpidr_el1", "Multiprocessor Affinity Register");
sysreg_read_func!(read_id_aa64mmfr0_el1, "id_aa64mmfr0_el1", "AArch64 Memory Model Feature Register 0");

sysreg_rw_func!(read_scr_el3, write_scr_el3, "scr_el3", "Secure Configuration Register");
sysreg_rw_func!(read_hcr_el2, write_hcr_el2, "hcr_el2", "Hypervisor Configuration Register");

sysreg_rw_func!(read_vbar_el1, write_vbar_el1, "vbar_el1", "Vector Base Address Register (EL1)");
sysreg_rw_func!(read_vbar_el2, write_vbar_el2, "vbar_el2", "Vector Base Address Register (EL2)");
sysreg_rw_func!(read_vbar_el3, write_vbar_el3, "vbar_el3", "Vector Base Address Register (EL3)");

sysreg_rw_func!(read_sctlr_el1, write_sctlr_el1, "sctlr_el1", "System Control Register (EL1)");
sysreg_rw_func!(read_sctlr_el2, write_sctlr_el2, "sctlr_el2", "System Control Register (EL2)");
sysreg_rw_func!(read_sctlr_el3, write_sctlr_el3, "sctlr_el3", "System Control Register (EL3)");

sysreg_rw_func!(read_actlr_el1, write_actlr_el1, "actlr_el1", "Auxiliary Control Register (EL1)");
sysreg_rw_func!(read_actlr_el2, write_actlr_el2, "actlr_el2", "Auxiliary Control Register (EL2)");
sysreg_rw_func!(read_actlr_el3, write_actlr_el3, "actlr_el3", "Auxiliary Control Register (EL3)");

sysreg_rw_func!(read_esr_el1, write_esr_el1, "esr_el1", "Exception Syndrome Register (EL1)");
sysreg_rw_func!(read_esr_el2, write_esr_el2, "esr_el2", "Exception Syndrome Register (EL2)");
sysreg_rw_func!(read_esr_el3, write_esr_el3, "esr_el3", "Exception Syndrome Register (EL3)");

sysreg_rw_func!(read_afsr0_el1, write_afsr0_el1, "afsr0_el1", "Auxiliary Fault Status Register 0 (EL1)");
sysreg_rw_func!(read_afsr0_el2, write_afsr0_el2, "afsr0_el2", "Auxiliary Fault Status Register 0 (EL2)");
sysreg_rw_func!(read_afsr0_el3, write_afsr0_el3, "afsr0_el3", "Auxiliary Fault Status Register 0 (EL3)");

sysreg_rw_func!(read_afsr1_el1, write_afsr1_el1, "afsr1_el1", "Auxiliary Fault Status Register 1 (EL1)");
sysreg_rw_func!(read_afsr1_el2, write_afsr1_el2, "afsr1_el2", "Auxiliary Fault Status Register 1 (EL2)");
sysreg_rw_func!(read_afsr1_el3, write_afsr1_el3, "afsr1_el3", "Auxiliary Fault Status Register 1 (EL3)");

sysreg_rw_func!(read_far_el1, write_far_el1, "far_el1", "Fault Address Register (EL1)");
sysreg_rw_func!(read_far_el2, write_far_el2, "far_el2", "Fault Address Register (EL2)");
sysreg_rw_func!(read_far_el3, write_far_el3, "far_el3", "Fault Address Register (EL3)");

sysreg_rw_func!(read_mair_el1, write_mair_el1, "mair_el1", "Memory Attribute Indirection Register (EL1)");
sysreg_rw_func!(read_mair_el2, write_mair_el2, "mair_el2", "Memory Attribute Indirection Register (EL2)");
sysreg_rw_func!(read_mair_el3, write_mair_el3, "mair_el3", "Memory Attribute Indirection Register (EL3)");

sysreg_rw_func!(read_amair_el1, write_amair_el1, "amair_el1", "Auxiliary Memory Attribute Indirection Register (EL1)");
sysreg_rw_func!(read_amair_el2, write_amair_el2, "amair_el2", "Auxiliary Memory Attribute Indirection Register (EL2)");
sysreg_rw_func!(read_amair_el3, write_amair_el3, "amair_el3", "Auxiliary Memory Attribute Indirection Register (EL3)");

sysreg_read_func!(read_rvbar_el1, "rvbar_el1", "Reset Vector Base Address Register (if EL2 and EL3 not implemented)");
sysreg_read_func!(read_rvbar_el2, "rvbar_el2", "RVBAR_EL2, Reset Vector Base Address Register (if EL3 not implemented)");
sysreg_read_func!(read_rvbar_el3, "rvbar_el3", "RVBAR_EL3, Reset Vector Base Address Register (if EL3 implemented)");

sysreg_rw_func!(read_rmr_el1, write_rmr_el1, "rmr_el1", "Reset Management Register (EL1)");
sysreg_rw_func!(read_rmr_el2, write_rmr_el2, "rmr_el2", "Reset Management Register (EL2)");
sysreg_rw_func!(read_rmr_el3, write_rmr_el3, "rmr_el3", "Reset Management Register (EL3)");

sysreg_rw_func!(read_tcr_el1, write_tcr_el1, "tcr_el1", "Translation Control Register (EL1)");
sysreg_rw_func!(read_tcr_el2, write_tcr_el2, "tcr_el2", "Translation Control Register (EL2)");
sysreg_rw_func!(read_tcr_el3, write_tcr_el3, "tcr_el3", "Translation Control Register (EL3)");

sysreg_rw_func!(read_ttbr0_el1, write_ttbr0_el1, "ttbr0_el1", "Translation Table Base Register 0 (EL1)");
sysreg_rw_func!(read_ttbr0_el2, write_ttbr0_el2, "ttbr0_el2", "Translation Table Base Register 0 (EL2)");
sysreg_rw_func!(read_ttbr0_el3, write_ttbr0_el3, "ttbr0_el3", "Translation Table Base Register 0 (EL3)");

sysreg_rw_func!(read_ttbr1_el1, write_ttbr1_el1, "ttbr1_el1", "Translation Table Base Register 1 (EL1)");

sysreg_rw_func!(read_vttbr_el2, write_vttbr_el2, "vttbr_el2", "Virtualization Translation Table Base Register");

sysreg_rw_func!(read_cptr_el2, write_cptr_el2, "cptr_el2", "Architectural Feature Trap Register (EL2)");
sysreg_rw_func!(read_cptr_el3, write_cptr_el3, "cptr_el3", "Architectural Feature Trap Register (EL3)");

sysreg_rw_func!(read_cpacr_el1, write_cpacr_el1, "cpacr_el1", "Architectural Feature Access Control Register");
sysreg_rw_func!(read_cntfrq_el0, write_cntfrq_el0, "cntfrq_el0", "Counter-timer Frequency register");
sysreg_rw_func!(read_cntps_ctl_el1, write_cntps_ctl_el1, "cntps_ctl_el1", "Counter-timer Physical Secure Timer Control register");
sysreg_rw_func!(read_cntps_tval_el1, write_cntps_tval_el1, "cntps_tval_el1", "Counter-timer Physical Secure Timer TimerValue register");
sysreg_rw_func!(read_cntps_cval_el1, write_cntps_cval_el1, "cntps_cval_el1", "Counter-timer Physical Secure Timer CompareValue register");
sysreg_read_func!(read_cntpct_el0,  "cntpct_el0", "Counter-timer Physical Count register");
sysreg_rw_func!(read_cnthctl_el2, write_cnthctl_el2, "cnthctl_el2", "Counter-timer Hypervisor Control register");

sysreg_rw_func!(read_tpidr_el3, write_tpidr_el3, "tpidr_el3", "EL3 Software Thread ID Register");

sysreg_rw_func!(read_cntvoff_el2, write_cntvoff_el2, "cntvoff_el2", "Counter-timer Virtual Offset register");

sysreg_rw_func!(read_vpidr_el2, write_vpidr_el2, "vpidr_el2", "Virtualization Processor ID Register");
sysreg_rw_func!(read_vmpidr_el2, write_vmpidr_el2, "vmpidr_el2", "Virtualization Multiprocessor ID Register");
sysreg_rw_func!(read_cntp_ctl_el0, write_cntp_ctl_el0, "cntp_ctl_el0", "Counter-timer Physical Timer Control register");

sysreg_read_func!(read_isr_el1,  "isr_el1", "Interrupt Status Register");

sysreg_read_func!(read_ctr_el0,  "ctr_el0", "Cache Type Register");

sysreg_rw_func!(read_mdcr_el2, write_mdcr_el2, "mdcr_el2", "Monitor Debug Configuration Register (EL2)");
sysreg_rw_func!(read_mdcr_el3, write_mdcr_el3, "mdcr_el3", "Monitor Debug Configuration Register (EL3)");
sysreg_rw_func!(read_hstr_el2, write_hstr_el2, "hstr_el2", "Hypervisor System Trap Register");
sysreg_rw_func!(read_cnthp_ctl_el2, write_cnthp_ctl_el2, "cnthp_ctl_el2", "Counter-timer Hypervisor Physical Timer Control register");
sysreg_rw_func!(read_pmcr_el0, write_pmcr_el0, "pmcr_el0", "Performance Monitors Control Register");

//TODO: dcsw functions

#[cfg(target_arch = "aarch64")]
fn dcache_line_size() -> u64 {
    let ctr_el0 = read_ctr_el0();
    4 << ((ctr_el0 >> 16) & ((1 << 4) - 1))
}

macro_rules! dcache_range_func {
    ( $name: ident, $op: ident, $desc: tt ) => {
        #[doc=$desc]
        #[cfg(target_arch = "aarch64")]
        pub fn $name(mut addr: u64, size: u64) {
            if size != 0 {
                let line_size = dcache_line_size();
                let end = addr + size;
                addr &= !(line_size - 1);
                while addr < end {
                    $op(addr);
                    addr += line_size;
                }
                dsb_sy();
            }
        }
    };
}

dcache_range_func!(flush_dcache_range, dc_civac, "Flush dcache range");
dcache_range_func!(clean_dcache_range, dc_cvac, "Clean dcache range");
dcache_range_func!(inv_dcache_range, dc_ivac, "Invalidate dcache range");


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
    ((read_id_aa64dfr0_el1() >> shift) & 0xF) != 0
}

// Previously defined accesor functions with incomplete register names

/// Read current exception level
#[cfg(target_arch = "aarch64")]
pub fn read_current_el() -> u64 {
    read_CurrentEl()
}

/// Data Synchronization Barrier
/// No instruction in program order after this instruction executes until this instruction completes.
#[cfg(target_arch = "aarch64")]
pub fn dsb() {
    dsb_sy()
}

/// Read Main ID Register
#[cfg(target_arch = "aarch64")]
pub fn read_midr() -> u64 {
    read_midr_el1()
}

/// Read Multiprocessor Affinity Register
#[cfg(target_arch = "aarch64")]
pub fn read_mpidr() -> u64 {
    read_mpidr_el1()
}

/// Read Secure Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn read_scr() -> u64 {
    read_scr_el3()
}

/// Write Secure Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn write_scr(v: u64) {
    write_scr_el3(v)
}

/// Read Hypervisor Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn read_hcr() -> u64 {
    read_hcr_el2()
}

/// Write Hypervisor Configuration Register
#[cfg(target_arch = "aarch64")]
pub fn write_hcr(v: u64) {
    write_hcr_el2(v)
}

///Read Architectural Feature Access Control Register
#[cfg(target_arch = "aarch64")]
pub fn read_cpacr() -> u64 {
    read_cpacr_el1()
}

///Write Architectural Feature Access Control Register
#[cfg(target_arch = "aarch64")]
pub fn write_cpacr(v: u64) {
    write_cpacr_el1(v)
}