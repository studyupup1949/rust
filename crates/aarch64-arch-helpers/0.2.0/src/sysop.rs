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
