/* GIC Distributor interface register offsets that are common to GICv3 & GICv2 */
pub const GICD_CTLR: u64 = 0x0;
pub const GICD_TYPER: u64 = 0x4;
pub const GICD_IIDR: u64 = 0x8;
pub const GICD_IGROUPR: u64 = 0x80;
pub const GICD_ISENABLER: u64 = 0x100;
pub const GICD_ICENABLER: u64 = 0x180;
pub const GICD_ISPENDR: u64 = 0x200;
pub const GICD_ICPENDR: u64 = 0x280;
pub const GICD_ISACTIVER: u64 = 0x300;
pub const GICD_ICACTIVER: u64 = 0x380;
pub const GICD_IPRIORITYR: u64 = 0x400;
pub const GICD_ITARGETSR: u64 = 0x800;
pub const GICD_ICFGR: u64 = 0xc00;
pub const GICD_NSACR: u64 = 0xe00;
pub const GICD_SGI: u64 = 0xF00;

/*#define GICD_CTLR_ENABLEGRP0		(1 << 0)
#define GICD_CTLR_ENABLEGRP1		(1 << 1)*/

/* Physical CPU Interface registers */
pub const GICC_CTLR: u64 = 0x0;
pub const GICC_PMR: u64 = 0x4;
pub const GICC_BPR: u64 = 0x8;
pub const GICC_IAR: u64 = 0xC;
pub const GICC_EOIR: u64 = 0x10;
pub const GICC_RPR: u64 = 0x14;
pub const GICC_HPPIR: u64 = 0x18;
pub const GICC_AHPPIR: u64 = 0x28;
pub const GICC_IIDR: u64 = 0xFC;
pub const GICC_DIR: u64 = 0x1000;
pub const GICC_PRIODROP: u64 = GICC_EOIR:

/*#define GICC_CTLR_ENABLEGRP0		(1 << 0)
#define GICC_CTLR_ENABLEGRP1		(1 << 1)
#define GICC_CTLR_FIQEN			(1 << 3)
#define GICC_CTLR_ACKCTL		(1 << 2)*/

// Representation of the Generic Interrupt Controller.
pub struct Gic {
}