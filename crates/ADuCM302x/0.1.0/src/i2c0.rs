#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Master Control"]
    pub mctl: MCTL,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - Master Status"]
    pub mstat: MSTAT,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Master Receive Data"]
    pub mrx: MRX,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Master Transmit Data"]
    pub mtx: MTX,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - Master Receive Data Count"]
    pub mrxcnt: MRXCNT,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - Master Current Receive Data Count"]
    pub mcrxcnt: MCRXCNT,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - Master Address Byte 1"]
    pub addr1: ADDR1,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - Master Address Byte 2"]
    pub addr2: ADDR2,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - Start Byte"]
    pub byt: BYT,
    _reserved9: [u8; 2usize],
    #[doc = "0x24 - Serial Clock Period Divisor"]
    pub div: DIV,
    _reserved10: [u8; 2usize],
    #[doc = "0x28 - Slave Control"]
    pub sctl: SCTL,
    _reserved11: [u8; 2usize],
    #[doc = "0x2c - Slave I2C Status/Error/IRQ"]
    pub sstat: SSTAT,
    _reserved12: [u8; 2usize],
    #[doc = "0x30 - Slave Receive"]
    pub srx: SRX,
    _reserved13: [u8; 2usize],
    #[doc = "0x34 - Slave Transmit"]
    pub stx: STX,
    _reserved14: [u8; 2usize],
    #[doc = "0x38 - Hardware General Call ID"]
    pub alt: ALT,
    _reserved15: [u8; 2usize],
    #[doc = "0x3c - First Slave Address Device ID"]
    pub id0: ID0,
    _reserved16: [u8; 2usize],
    #[doc = "0x40 - Second Slave Address Device ID"]
    pub id1: ID1,
    _reserved17: [u8; 2usize],
    #[doc = "0x44 - Third Slave Address Device ID"]
    pub id2: ID2,
    _reserved18: [u8; 2usize],
    #[doc = "0x48 - Fourth Slave Address Device ID"]
    pub id3: ID3,
    _reserved19: [u8; 2usize],
    #[doc = "0x4c - Master and Slave FIFO Status"]
    pub stat: STAT,
    _reserved20: [u8; 2usize],
    #[doc = "0x50 - Shared Control"]
    pub shctl: SHCTL,
    _reserved21: [u8; 2usize],
    #[doc = "0x54 - Timing Control Register"]
    pub tctl: TCTL,
    _reserved22: [u8; 2usize],
    #[doc = "0x58 - Automatic Stretch SCL"]
    pub astretch_scl: ASTRETCH_SCL,
}
#[doc = "Master Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mctl](mctl) module"]
pub type MCTL = crate::Reg<u16, _MCTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MCTL;
#[doc = "`read()` method returns [mctl::R](mctl::R) reader structure"]
impl crate::Readable for MCTL {}
#[doc = "`write(|w| ..)` method takes [mctl::W](mctl::W) writer structure"]
impl crate::Writable for MCTL {}
#[doc = "Master Control"]
pub mod mctl;
#[doc = "Master Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mstat](mstat) module"]
pub type MSTAT = crate::Reg<u16, _MSTAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MSTAT;
#[doc = "`read()` method returns [mstat::R](mstat::R) reader structure"]
impl crate::Readable for MSTAT {}
#[doc = "`write(|w| ..)` method takes [mstat::W](mstat::W) writer structure"]
impl crate::Writable for MSTAT {}
#[doc = "Master Status"]
pub mod mstat;
#[doc = "Master Receive Data\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mrx](mrx) module"]
pub type MRX = crate::Reg<u16, _MRX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MRX;
#[doc = "`read()` method returns [mrx::R](mrx::R) reader structure"]
impl crate::Readable for MRX {}
#[doc = "Master Receive Data"]
pub mod mrx;
#[doc = "Master Transmit Data\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mtx](mtx) module"]
pub type MTX = crate::Reg<u16, _MTX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MTX;
#[doc = "`read()` method returns [mtx::R](mtx::R) reader structure"]
impl crate::Readable for MTX {}
#[doc = "`write(|w| ..)` method takes [mtx::W](mtx::W) writer structure"]
impl crate::Writable for MTX {}
#[doc = "Master Transmit Data"]
pub mod mtx;
#[doc = "Master Receive Data Count\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mrxcnt](mrxcnt) module"]
pub type MRXCNT = crate::Reg<u16, _MRXCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MRXCNT;
#[doc = "`read()` method returns [mrxcnt::R](mrxcnt::R) reader structure"]
impl crate::Readable for MRXCNT {}
#[doc = "`write(|w| ..)` method takes [mrxcnt::W](mrxcnt::W) writer structure"]
impl crate::Writable for MRXCNT {}
#[doc = "Master Receive Data Count"]
pub mod mrxcnt;
#[doc = "Master Current Receive Data Count\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mcrxcnt](mcrxcnt) module"]
pub type MCRXCNT = crate::Reg<u16, _MCRXCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MCRXCNT;
#[doc = "`read()` method returns [mcrxcnt::R](mcrxcnt::R) reader structure"]
impl crate::Readable for MCRXCNT {}
#[doc = "Master Current Receive Data Count"]
pub mod mcrxcnt;
#[doc = "Master Address Byte 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [addr1](addr1) module"]
pub type ADDR1 = crate::Reg<u16, _ADDR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ADDR1;
#[doc = "`read()` method returns [addr1::R](addr1::R) reader structure"]
impl crate::Readable for ADDR1 {}
#[doc = "`write(|w| ..)` method takes [addr1::W](addr1::W) writer structure"]
impl crate::Writable for ADDR1 {}
#[doc = "Master Address Byte 1"]
pub mod addr1;
#[doc = "Master Address Byte 2\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [addr2](addr2) module"]
pub type ADDR2 = crate::Reg<u16, _ADDR2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ADDR2;
#[doc = "`read()` method returns [addr2::R](addr2::R) reader structure"]
impl crate::Readable for ADDR2 {}
#[doc = "`write(|w| ..)` method takes [addr2::W](addr2::W) writer structure"]
impl crate::Writable for ADDR2 {}
#[doc = "Master Address Byte 2"]
pub mod addr2;
#[doc = "Start Byte\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [byt](byt) module"]
pub type BYT = crate::Reg<u16, _BYT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _BYT;
#[doc = "`read()` method returns [byt::R](byt::R) reader structure"]
impl crate::Readable for BYT {}
#[doc = "`write(|w| ..)` method takes [byt::W](byt::W) writer structure"]
impl crate::Writable for BYT {}
#[doc = "Start Byte"]
pub mod byt;
#[doc = "Serial Clock Period Divisor\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [div](div) module"]
pub type DIV = crate::Reg<u16, _DIV>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DIV;
#[doc = "`read()` method returns [div::R](div::R) reader structure"]
impl crate::Readable for DIV {}
#[doc = "`write(|w| ..)` method takes [div::W](div::W) writer structure"]
impl crate::Writable for DIV {}
#[doc = "Serial Clock Period Divisor"]
pub mod div;
#[doc = "Slave Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sctl](sctl) module"]
pub type SCTL = crate::Reg<u16, _SCTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SCTL;
#[doc = "`read()` method returns [sctl::R](sctl::R) reader structure"]
impl crate::Readable for SCTL {}
#[doc = "`write(|w| ..)` method takes [sctl::W](sctl::W) writer structure"]
impl crate::Writable for SCTL {}
#[doc = "Slave Control"]
pub mod sctl;
#[doc = "Slave I2C Status/Error/IRQ\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sstat](sstat) module"]
pub type SSTAT = crate::Reg<u16, _SSTAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SSTAT;
#[doc = "`read()` method returns [sstat::R](sstat::R) reader structure"]
impl crate::Readable for SSTAT {}
#[doc = "`write(|w| ..)` method takes [sstat::W](sstat::W) writer structure"]
impl crate::Writable for SSTAT {}
#[doc = "Slave I2C Status/Error/IRQ"]
pub mod sstat;
#[doc = "Slave Receive\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [srx](srx) module"]
pub type SRX = crate::Reg<u16, _SRX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRX;
#[doc = "`read()` method returns [srx::R](srx::R) reader structure"]
impl crate::Readable for SRX {}
#[doc = "Slave Receive"]
pub mod srx;
#[doc = "Slave Transmit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stx](stx) module"]
pub type STX = crate::Reg<u16, _STX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STX;
#[doc = "`read()` method returns [stx::R](stx::R) reader structure"]
impl crate::Readable for STX {}
#[doc = "`write(|w| ..)` method takes [stx::W](stx::W) writer structure"]
impl crate::Writable for STX {}
#[doc = "Slave Transmit"]
pub mod stx;
#[doc = "Hardware General Call ID\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alt](alt) module"]
pub type ALT = crate::Reg<u16, _ALT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALT;
#[doc = "`read()` method returns [alt::R](alt::R) reader structure"]
impl crate::Readable for ALT {}
#[doc = "`write(|w| ..)` method takes [alt::W](alt::W) writer structure"]
impl crate::Writable for ALT {}
#[doc = "Hardware General Call ID"]
pub mod alt;
#[doc = "First Slave Address Device ID\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [id0](id0) module"]
pub type ID0 = crate::Reg<u16, _ID0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ID0;
#[doc = "`read()` method returns [id0::R](id0::R) reader structure"]
impl crate::Readable for ID0 {}
#[doc = "`write(|w| ..)` method takes [id0::W](id0::W) writer structure"]
impl crate::Writable for ID0 {}
#[doc = "First Slave Address Device ID"]
pub mod id0;
#[doc = "Second Slave Address Device ID\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [id1](id1) module"]
pub type ID1 = crate::Reg<u16, _ID1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ID1;
#[doc = "`read()` method returns [id1::R](id1::R) reader structure"]
impl crate::Readable for ID1 {}
#[doc = "`write(|w| ..)` method takes [id1::W](id1::W) writer structure"]
impl crate::Writable for ID1 {}
#[doc = "Second Slave Address Device ID"]
pub mod id1;
#[doc = "Third Slave Address Device ID\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [id2](id2) module"]
pub type ID2 = crate::Reg<u16, _ID2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ID2;
#[doc = "`read()` method returns [id2::R](id2::R) reader structure"]
impl crate::Readable for ID2 {}
#[doc = "`write(|w| ..)` method takes [id2::W](id2::W) writer structure"]
impl crate::Writable for ID2 {}
#[doc = "Third Slave Address Device ID"]
pub mod id2;
#[doc = "Fourth Slave Address Device ID\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [id3](id3) module"]
pub type ID3 = crate::Reg<u16, _ID3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ID3;
#[doc = "`read()` method returns [id3::R](id3::R) reader structure"]
impl crate::Readable for ID3 {}
#[doc = "`write(|w| ..)` method takes [id3::W](id3::W) writer structure"]
impl crate::Writable for ID3 {}
#[doc = "Fourth Slave Address Device ID"]
pub mod id3;
#[doc = "Master and Slave FIFO Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "Master and Slave FIFO Status"]
pub mod stat;
#[doc = "Shared Control\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shctl](shctl) module"]
pub type SHCTL = crate::Reg<u16, _SHCTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHCTL;
#[doc = "`write(|w| ..)` method takes [shctl::W](shctl::W) writer structure"]
impl crate::Writable for SHCTL {}
#[doc = "Shared Control"]
pub mod shctl;
#[doc = "Timing Control Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tctl](tctl) module"]
pub type TCTL = crate::Reg<u16, _TCTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TCTL;
#[doc = "`read()` method returns [tctl::R](tctl::R) reader structure"]
impl crate::Readable for TCTL {}
#[doc = "`write(|w| ..)` method takes [tctl::W](tctl::W) writer structure"]
impl crate::Writable for TCTL {}
#[doc = "Timing Control Register"]
pub mod tctl;
#[doc = "Automatic Stretch SCL\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [astretch_scl](astretch_scl) module"]
pub type ASTRETCH_SCL = crate::Reg<u16, _ASTRETCH_SCL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ASTRETCH_SCL;
#[doc = "`read()` method returns [astretch_scl::R](astretch_scl::R) reader structure"]
impl crate::Readable for ASTRETCH_SCL {}
#[doc = "`write(|w| ..)` method takes [astretch_scl::W](astretch_scl::W) writer structure"]
impl crate::Writable for ASTRETCH_SCL {}
#[doc = "Automatic Stretch SCL"]
pub mod astretch_scl;
