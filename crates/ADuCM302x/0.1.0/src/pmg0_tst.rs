#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved0: [u8; 96usize],
    #[doc = "0x60 - Control for SRAM Parity and Instruction SRAM"]
    pub sram_ctl: SRAM_CTL,
    #[doc = "0x64 - Initialization Status Register"]
    pub sram_initstat: SRAM_INITSTAT,
    #[doc = "0x68 - Clear GPIO After Shutdown Mode"]
    pub clr_latch_gpios: CLR_LATCH_GPIOS,
    _reserved3: [u8; 2usize],
    #[doc = "0x6c - Scratch Pad Image"]
    pub scrpad_img: SCRPAD_IMG,
    #[doc = "0x70 - Scratch Pad Saved in Battery Domain"]
    pub scrpad_3v_rd: SCRPAD_3V_RD,
}
#[doc = "Control for SRAM Parity and Instruction SRAM\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sram_ctl](sram_ctl) module"]
pub type SRAM_CTL = crate::Reg<u32, _SRAM_CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRAM_CTL;
#[doc = "`read()` method returns [sram_ctl::R](sram_ctl::R) reader structure"]
impl crate::Readable for SRAM_CTL {}
#[doc = "`write(|w| ..)` method takes [sram_ctl::W](sram_ctl::W) writer structure"]
impl crate::Writable for SRAM_CTL {}
#[doc = "Control for SRAM Parity and Instruction SRAM"]
pub mod sram_ctl;
#[doc = "Initialization Status Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sram_initstat](sram_initstat) module"]
pub type SRAM_INITSTAT = crate::Reg<u32, _SRAM_INITSTAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRAM_INITSTAT;
#[doc = "`read()` method returns [sram_initstat::R](sram_initstat::R) reader structure"]
impl crate::Readable for SRAM_INITSTAT {}
#[doc = "Initialization Status Register"]
pub mod sram_initstat;
#[doc = "Clear GPIO After Shutdown Mode\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [clr_latch_gpios](clr_latch_gpios) module"]
pub type CLR_LATCH_GPIOS = crate::Reg<u16, _CLR_LATCH_GPIOS>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CLR_LATCH_GPIOS;
#[doc = "`write(|w| ..)` method takes [clr_latch_gpios::W](clr_latch_gpios::W) writer structure"]
impl crate::Writable for CLR_LATCH_GPIOS {}
#[doc = "Clear GPIO After Shutdown Mode"]
pub mod clr_latch_gpios;
#[doc = "Scratch Pad Image\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [scrpad_img](scrpad_img) module"]
pub type SCRPAD_IMG = crate::Reg<u32, _SCRPAD_IMG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SCRPAD_IMG;
#[doc = "`read()` method returns [scrpad_img::R](scrpad_img::R) reader structure"]
impl crate::Readable for SCRPAD_IMG {}
#[doc = "`write(|w| ..)` method takes [scrpad_img::W](scrpad_img::W) writer structure"]
impl crate::Writable for SCRPAD_IMG {}
#[doc = "Scratch Pad Image"]
pub mod scrpad_img;
#[doc = "Scratch Pad Saved in Battery Domain\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [scrpad_3v_rd](scrpad_3v_rd) module"]
pub type SCRPAD_3V_RD = crate::Reg<u32, _SCRPAD_3V_RD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SCRPAD_3V_RD;
#[doc = "`read()` method returns [scrpad_3v_rd::R](scrpad_3v_rd::R) reader structure"]
impl crate::Readable for SCRPAD_3V_RD {}
#[doc = "Scratch Pad Saved in Battery Domain"]
pub mod scrpad_3v_rd;
