#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Power Supply Monitor Interrupt Enable"]
    pub ien: IEN,
    #[doc = "0x04 - Power Supply Monitor Status"]
    pub psm_stat: PSM_STAT,
    #[doc = "0x08 - Power Mode Register"]
    pub pwrmod: PWRMOD,
    #[doc = "0x0c - Key Protection for PMG_PWRMOD and PMG_SRAMRET"]
    pub pwrkey: PWRKEY,
    #[doc = "0x10 - Shutdown Status Register"]
    pub shdn_stat: SHDN_STAT,
    #[doc = "0x14 - Control for Retention SRAM in Hibernate Mode"]
    pub sramret: SRAMRET,
    _reserved6: [u8; 40usize],
    #[doc = "0x40 - Reset Status"]
    pub rst_stat: RST_STAT,
    #[doc = "0x44 - HP Buck Control"]
    pub ctl1: CTL1,
}
#[doc = "Power Supply Monitor Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien](ien) module"]
pub type IEN = crate::Reg<u32, _IEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN;
#[doc = "`read()` method returns [ien::R](ien::R) reader structure"]
impl crate::Readable for IEN {}
#[doc = "`write(|w| ..)` method takes [ien::W](ien::W) writer structure"]
impl crate::Writable for IEN {}
#[doc = "Power Supply Monitor Interrupt Enable"]
pub mod ien;
#[doc = "Power Supply Monitor Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [psm_stat](psm_stat) module"]
pub type PSM_STAT = crate::Reg<u32, _PSM_STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PSM_STAT;
#[doc = "`read()` method returns [psm_stat::R](psm_stat::R) reader structure"]
impl crate::Readable for PSM_STAT {}
#[doc = "`write(|w| ..)` method takes [psm_stat::W](psm_stat::W) writer structure"]
impl crate::Writable for PSM_STAT {}
#[doc = "Power Supply Monitor Status"]
pub mod psm_stat;
#[doc = "Power Mode Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pwrmod](pwrmod) module"]
pub type PWRMOD = crate::Reg<u32, _PWRMOD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PWRMOD;
#[doc = "`read()` method returns [pwrmod::R](pwrmod::R) reader structure"]
impl crate::Readable for PWRMOD {}
#[doc = "`write(|w| ..)` method takes [pwrmod::W](pwrmod::W) writer structure"]
impl crate::Writable for PWRMOD {}
#[doc = "Power Mode Register"]
pub mod pwrmod;
#[doc = "Key Protection for PMG_PWRMOD and PMG_SRAMRET\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pwrkey](pwrkey) module"]
pub type PWRKEY = crate::Reg<u32, _PWRKEY>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PWRKEY;
#[doc = "`write(|w| ..)` method takes [pwrkey::W](pwrkey::W) writer structure"]
impl crate::Writable for PWRKEY {}
#[doc = "Key Protection for PMG_PWRMOD and PMG_SRAMRET"]
pub mod pwrkey;
#[doc = "Shutdown Status Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shdn_stat](shdn_stat) module"]
pub type SHDN_STAT = crate::Reg<u32, _SHDN_STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHDN_STAT;
#[doc = "`read()` method returns [shdn_stat::R](shdn_stat::R) reader structure"]
impl crate::Readable for SHDN_STAT {}
#[doc = "Shutdown Status Register"]
pub mod shdn_stat;
#[doc = "Control for Retention SRAM in Hibernate Mode\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sramret](sramret) module"]
pub type SRAMRET = crate::Reg<u32, _SRAMRET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRAMRET;
#[doc = "`read()` method returns [sramret::R](sramret::R) reader structure"]
impl crate::Readable for SRAMRET {}
#[doc = "`write(|w| ..)` method takes [sramret::W](sramret::W) writer structure"]
impl crate::Writable for SRAMRET {}
#[doc = "Control for Retention SRAM in Hibernate Mode"]
pub mod sramret;
#[doc = "Reset Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rst_stat](rst_stat) module"]
pub type RST_STAT = crate::Reg<u32, _RST_STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RST_STAT;
#[doc = "`read()` method returns [rst_stat::R](rst_stat::R) reader structure"]
impl crate::Readable for RST_STAT {}
#[doc = "`write(|w| ..)` method takes [rst_stat::W](rst_stat::W) writer structure"]
impl crate::Writable for RST_STAT {}
#[doc = "Reset Status"]
pub mod rst_stat;
#[doc = "HP Buck Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl1](ctl1) module"]
pub type CTL1 = crate::Reg<u32, _CTL1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL1;
#[doc = "`read()` method returns [ctl1::R](ctl1::R) reader structure"]
impl crate::Readable for CTL1 {}
#[doc = "`write(|w| ..)` method takes [ctl1::W](ctl1::W) writer structure"]
impl crate::Writable for CTL1 {}
#[doc = "HP Buck Control"]
pub mod ctl1;
