#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Half SPORT 'A' Control"]
    pub ctl_a: CTL_A,
    #[doc = "0x04 - Half SPORT 'A' Divisor"]
    pub div_a: DIV_A,
    #[doc = "0x08 - Half SPORT A's Interrupt Enable"]
    pub ien_a: IEN_A,
    #[doc = "0x0c - Half SPORT A's Status"]
    pub stat_a: STAT_A,
    #[doc = "0x10 - Half SPORT A Number of Transfers"]
    pub numtran_a: NUMTRAN_A,
    #[doc = "0x14 - Half SPORT 'A' CNV Width"]
    pub cnvt_a: CNVT_A,
    _reserved6: [u8; 8usize],
    #[doc = "0x20 - Half SPORT 'A' Tx Buffer"]
    pub tx_a: TX_A,
    _reserved7: [u8; 4usize],
    #[doc = "0x28 - Half SPORT 'A' Rx Buffer"]
    pub rx_a: RX_A,
    _reserved8: [u8; 20usize],
    #[doc = "0x40 - Half SPORT 'B' Control"]
    pub ctl_b: CTL_B,
    #[doc = "0x44 - Half SPORT 'B' Divisor"]
    pub div_b: DIV_B,
    #[doc = "0x48 - Half SPORT B's Interrupt Enable"]
    pub ien_b: IEN_B,
    #[doc = "0x4c - Half SPORT B's Status"]
    pub stat_b: STAT_B,
    #[doc = "0x50 - Half SPORT B Number of Transfers"]
    pub numtran_b: NUMTRAN_B,
    #[doc = "0x54 - Half SPORT 'B' CNV Width"]
    pub cnvt_b: CNVT_B,
    _reserved14: [u8; 8usize],
    #[doc = "0x60 - Half SPORT 'B' Tx Buffer"]
    pub tx_b: TX_B,
    _reserved15: [u8; 4usize],
    #[doc = "0x68 - Half SPORT 'B' Rx Buffer"]
    pub rx_b: RX_B,
}
#[doc = "Half SPORT 'A' Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl_a](ctl_a) module"]
pub type CTL_A = crate::Reg<u32, _CTL_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL_A;
#[doc = "`read()` method returns [ctl_a::R](ctl_a::R) reader structure"]
impl crate::Readable for CTL_A {}
#[doc = "`write(|w| ..)` method takes [ctl_a::W](ctl_a::W) writer structure"]
impl crate::Writable for CTL_A {}
#[doc = "Half SPORT 'A' Control"]
pub mod ctl_a;
#[doc = "Half SPORT 'A' Divisor\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [div_a](div_a) module"]
pub type DIV_A = crate::Reg<u32, _DIV_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DIV_A;
#[doc = "`read()` method returns [div_a::R](div_a::R) reader structure"]
impl crate::Readable for DIV_A {}
#[doc = "`write(|w| ..)` method takes [div_a::W](div_a::W) writer structure"]
impl crate::Writable for DIV_A {}
#[doc = "Half SPORT 'A' Divisor"]
pub mod div_a;
#[doc = "Half SPORT A's Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien_a](ien_a) module"]
pub type IEN_A = crate::Reg<u32, _IEN_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN_A;
#[doc = "`read()` method returns [ien_a::R](ien_a::R) reader structure"]
impl crate::Readable for IEN_A {}
#[doc = "`write(|w| ..)` method takes [ien_a::W](ien_a::W) writer structure"]
impl crate::Writable for IEN_A {}
#[doc = "Half SPORT A's Interrupt Enable"]
pub mod ien_a;
#[doc = "Half SPORT A's Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat_a](stat_a) module"]
pub type STAT_A = crate::Reg<u32, _STAT_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT_A;
#[doc = "`read()` method returns [stat_a::R](stat_a::R) reader structure"]
impl crate::Readable for STAT_A {}
#[doc = "Half SPORT A's Status"]
pub mod stat_a;
#[doc = "Half SPORT A Number of Transfers\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [numtran_a](numtran_a) module"]
pub type NUMTRAN_A = crate::Reg<u32, _NUMTRAN_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NUMTRAN_A;
#[doc = "`read()` method returns [numtran_a::R](numtran_a::R) reader structure"]
impl crate::Readable for NUMTRAN_A {}
#[doc = "`write(|w| ..)` method takes [numtran_a::W](numtran_a::W) writer structure"]
impl crate::Writable for NUMTRAN_A {}
#[doc = "Half SPORT A Number of Transfers"]
pub mod numtran_a;
#[doc = "Half SPORT 'A' CNV Width\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnvt_a](cnvt_a) module"]
pub type CNVT_A = crate::Reg<u32, _CNVT_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNVT_A;
#[doc = "`read()` method returns [cnvt_a::R](cnvt_a::R) reader structure"]
impl crate::Readable for CNVT_A {}
#[doc = "`write(|w| ..)` method takes [cnvt_a::W](cnvt_a::W) writer structure"]
impl crate::Writable for CNVT_A {}
#[doc = "Half SPORT 'A' CNV Width"]
pub mod cnvt_a;
#[doc = "Half SPORT 'A' Tx Buffer\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tx_a](tx_a) module"]
pub type TX_A = crate::Reg<u32, _TX_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TX_A;
#[doc = "`write(|w| ..)` method takes [tx_a::W](tx_a::W) writer structure"]
impl crate::Writable for TX_A {}
#[doc = "Half SPORT 'A' Tx Buffer"]
pub mod tx_a;
#[doc = "Half SPORT 'A' Rx Buffer\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rx_a](rx_a) module"]
pub type RX_A = crate::Reg<u32, _RX_A>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RX_A;
#[doc = "`read()` method returns [rx_a::R](rx_a::R) reader structure"]
impl crate::Readable for RX_A {}
#[doc = "Half SPORT 'A' Rx Buffer"]
pub mod rx_a;
#[doc = "Half SPORT 'B' Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl_b](ctl_b) module"]
pub type CTL_B = crate::Reg<u32, _CTL_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL_B;
#[doc = "`read()` method returns [ctl_b::R](ctl_b::R) reader structure"]
impl crate::Readable for CTL_B {}
#[doc = "`write(|w| ..)` method takes [ctl_b::W](ctl_b::W) writer structure"]
impl crate::Writable for CTL_B {}
#[doc = "Half SPORT 'B' Control"]
pub mod ctl_b;
#[doc = "Half SPORT 'B' Divisor\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [div_b](div_b) module"]
pub type DIV_B = crate::Reg<u32, _DIV_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DIV_B;
#[doc = "`read()` method returns [div_b::R](div_b::R) reader structure"]
impl crate::Readable for DIV_B {}
#[doc = "`write(|w| ..)` method takes [div_b::W](div_b::W) writer structure"]
impl crate::Writable for DIV_B {}
#[doc = "Half SPORT 'B' Divisor"]
pub mod div_b;
#[doc = "Half SPORT B's Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien_b](ien_b) module"]
pub type IEN_B = crate::Reg<u32, _IEN_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN_B;
#[doc = "`read()` method returns [ien_b::R](ien_b::R) reader structure"]
impl crate::Readable for IEN_B {}
#[doc = "`write(|w| ..)` method takes [ien_b::W](ien_b::W) writer structure"]
impl crate::Writable for IEN_B {}
#[doc = "Half SPORT B's Interrupt Enable"]
pub mod ien_b;
#[doc = "Half SPORT B's Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat_b](stat_b) module"]
pub type STAT_B = crate::Reg<u32, _STAT_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT_B;
#[doc = "`read()` method returns [stat_b::R](stat_b::R) reader structure"]
impl crate::Readable for STAT_B {}
#[doc = "Half SPORT B's Status"]
pub mod stat_b;
#[doc = "Half SPORT B Number of Transfers\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [numtran_b](numtran_b) module"]
pub type NUMTRAN_B = crate::Reg<u32, _NUMTRAN_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NUMTRAN_B;
#[doc = "`read()` method returns [numtran_b::R](numtran_b::R) reader structure"]
impl crate::Readable for NUMTRAN_B {}
#[doc = "`write(|w| ..)` method takes [numtran_b::W](numtran_b::W) writer structure"]
impl crate::Writable for NUMTRAN_B {}
#[doc = "Half SPORT B Number of Transfers"]
pub mod numtran_b;
#[doc = "Half SPORT 'B' CNV Width\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnvt_b](cnvt_b) module"]
pub type CNVT_B = crate::Reg<u32, _CNVT_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNVT_B;
#[doc = "`read()` method returns [cnvt_b::R](cnvt_b::R) reader structure"]
impl crate::Readable for CNVT_B {}
#[doc = "`write(|w| ..)` method takes [cnvt_b::W](cnvt_b::W) writer structure"]
impl crate::Writable for CNVT_B {}
#[doc = "Half SPORT 'B' CNV Width"]
pub mod cnvt_b;
#[doc = "Half SPORT 'B' Tx Buffer\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tx_b](tx_b) module"]
pub type TX_B = crate::Reg<u32, _TX_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TX_B;
#[doc = "`write(|w| ..)` method takes [tx_b::W](tx_b::W) writer structure"]
impl crate::Writable for TX_B {}
#[doc = "Half SPORT 'B' Tx Buffer"]
pub mod tx_b;
#[doc = "Half SPORT 'B' Rx Buffer\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rx_b](rx_b) module"]
pub type RX_B = crate::Reg<u32, _RX_B>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RX_B;
#[doc = "`read()` method returns [rx_b::R](rx_b::R) reader structure"]
impl crate::Readable for RX_B {}
#[doc = "Half SPORT 'B' Rx Buffer"]
pub mod rx_b;
