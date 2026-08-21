#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Beeper Configuration"]
    pub cfg: CFG,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - Beeper Status"]
    pub stat: STAT,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Tone A Data"]
    pub tonea: TONEA,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Tone B Data"]
    pub toneb: TONEB,
}
#[doc = "Beeper Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub type CFG = crate::Reg<u16, _CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG;
#[doc = "`read()` method returns [cfg::R](cfg::R) reader structure"]
impl crate::Readable for CFG {}
#[doc = "`write(|w| ..)` method takes [cfg::W](cfg::W) writer structure"]
impl crate::Writable for CFG {}
#[doc = "Beeper Configuration"]
pub mod cfg;
#[doc = "Beeper Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "Beeper Status"]
pub mod stat;
#[doc = "Tone A Data\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tonea](tonea) module"]
pub type TONEA = crate::Reg<u16, _TONEA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TONEA;
#[doc = "`read()` method returns [tonea::R](tonea::R) reader structure"]
impl crate::Readable for TONEA {}
#[doc = "`write(|w| ..)` method takes [tonea::W](tonea::W) writer structure"]
impl crate::Writable for TONEA {}
#[doc = "Tone A Data"]
pub mod tonea;
#[doc = "Tone B Data\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [toneb](toneb) module"]
pub type TONEB = crate::Reg<u16, _TONEB>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TONEB;
#[doc = "`read()` method returns [toneb::R](toneb::R) reader structure"]
impl crate::Readable for TONEB {}
#[doc = "`write(|w| ..)` method takes [toneb::W](toneb::W) writer structure"]
impl crate::Writable for TONEB {}
#[doc = "Tone B Data"]
pub mod toneb;
