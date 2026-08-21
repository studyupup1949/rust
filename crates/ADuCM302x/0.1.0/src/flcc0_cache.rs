#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Cache Status"]
    pub stat: STAT,
    #[doc = "0x04 - Cache Setup"]
    pub setup: SETUP,
    #[doc = "0x08 - Cache Key"]
    pub key: KEY,
}
#[doc = "Cache Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u32, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "Cache Status"]
pub mod stat;
#[doc = "Cache Setup\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [setup](setup) module"]
pub type SETUP = crate::Reg<u32, _SETUP>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SETUP;
#[doc = "`read()` method returns [setup::R](setup::R) reader structure"]
impl crate::Readable for SETUP {}
#[doc = "`write(|w| ..)` method takes [setup::W](setup::W) writer structure"]
impl crate::Writable for SETUP {}
#[doc = "Cache Setup"]
pub mod setup;
#[doc = "Cache Key\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [key](key) module"]
pub type KEY = crate::Reg<u32, _KEY>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _KEY;
#[doc = "`write(|w| ..)` method takes [key::W](key::W) writer structure"]
impl crate::Writable for KEY {}
#[doc = "Cache Key"]
pub mod key;
