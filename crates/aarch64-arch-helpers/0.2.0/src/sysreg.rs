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
