#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved0: [u8; 32usize],
    #[doc = "0x20 - ADI Identification"]
    pub adiid: ADIID,
    _reserved1: [u8; 2usize],
    #[doc = "0x24 - Chip Identifier"]
    pub chipid: CHIPID,
    _reserved2: [u8; 26usize],
    #[doc = "0x40 - Serial Wire Debug Enable"]
    pub swden: SWDEN,
}
#[doc = "ADI Identification\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [adiid](adiid) module"]
pub type ADIID = crate::Reg<u16, _ADIID>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ADIID;
#[doc = "`read()` method returns [adiid::R](adiid::R) reader structure"]
impl crate::Readable for ADIID {}
#[doc = "ADI Identification"]
pub mod adiid;
#[doc = "Chip Identifier\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [chipid](chipid) module"]
pub type CHIPID = crate::Reg<u16, _CHIPID>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CHIPID;
#[doc = "`read()` method returns [chipid::R](chipid::R) reader structure"]
impl crate::Readable for CHIPID {}
#[doc = "Chip Identifier"]
pub mod chipid;
#[doc = "Serial Wire Debug Enable\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [swden](swden) module"]
pub type SWDEN = crate::Reg<u16, _SWDEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SWDEN;
#[doc = "`write(|w| ..)` method takes [swden::W](swden::W) writer structure"]
impl crate::Writable for SWDEN {}
#[doc = "Serial Wire Debug Enable"]
pub mod swden;
