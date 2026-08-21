#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - External Interrupt Configuration"]
    pub cfg0: CFG0,
    #[doc = "0x04 - External Wakeup Interrupt Status"]
    pub ext_stat: EXT_STAT,
    _reserved2: [u8; 8usize],
    #[doc = "0x10 - External Interrupt Clear"]
    pub clr: CLR,
    #[doc = "0x14 - Non-Maskable Interrupt Clear"]
    pub nmiclr: NMICLR,
}
#[doc = "External Interrupt Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cfg0](cfg0) module"]
pub type CFG0 = crate::Reg<u32, _CFG0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG0;
#[doc = "`read()` method returns [cfg0::R](cfg0::R) reader structure"]
impl crate::Readable for CFG0 {}
#[doc = "`write(|w| ..)` method takes [cfg0::W](cfg0::W) writer structure"]
impl crate::Writable for CFG0 {}
#[doc = "External Interrupt Configuration"]
pub mod cfg0;
#[doc = "External Wakeup Interrupt Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ext_stat](ext_stat) module"]
pub type EXT_STAT = crate::Reg<u32, _EXT_STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _EXT_STAT;
#[doc = "`read()` method returns [ext_stat::R](ext_stat::R) reader structure"]
impl crate::Readable for EXT_STAT {}
#[doc = "External Wakeup Interrupt Status"]
pub mod ext_stat;
#[doc = "External Interrupt Clear\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [clr](clr) module"]
pub type CLR = crate::Reg<u32, _CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CLR;
#[doc = "`read()` method returns [clr::R](clr::R) reader structure"]
impl crate::Readable for CLR {}
#[doc = "`write(|w| ..)` method takes [clr::W](clr::W) writer structure"]
impl crate::Writable for CLR {}
#[doc = "External Interrupt Clear"]
pub mod clr;
#[doc = "Non-Maskable Interrupt Clear\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [nmiclr](nmiclr) module"]
pub type NMICLR = crate::Reg<u32, _NMICLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NMICLR;
#[doc = "`read()` method returns [nmiclr::R](nmiclr::R) reader structure"]
impl crate::Readable for NMICLR {}
#[doc = "`write(|w| ..)` method takes [nmiclr::W](nmiclr::W) writer structure"]
impl crate::Writable for NMICLR {}
#[doc = "Non-Maskable Interrupt Clear"]
pub mod nmiclr;
