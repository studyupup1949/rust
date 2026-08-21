use crate::sysop::{dc_civac, dc_cvac, dc_ivac, dsb_sy};
use crate::sysreg::read_ctr_el0;

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

#[cfg(target_arch = "aarch64")]
fn dcache_line_size() -> u64 {
    let ctr_el0 = read_ctr_el0();
    4 << ((ctr_el0 >> 16) & ((1 << 4) - 1))
}

dcache_range_func!(flush_dcache_range, dc_civac, "Flush dcache range");
dcache_range_func!(clean_dcache_range, dc_cvac, "Clean dcache range");
dcache_range_func!(inv_dcache_range, dc_ivac, "Invalidate dcache range");
