#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Status"]
    pub stat: STAT,
    #[doc = "0x04 - Interrupt Enable"]
    pub ien: IEN,
    #[doc = "0x08 - Command"]
    pub cmd: CMD,
    #[doc = "0x0c - Write Address"]
    pub kh_addr: KH_ADDR,
    #[doc = "0x10 - Write Lower Data"]
    pub kh_data0: KH_DATA0,
    #[doc = "0x14 - Write Upper Data"]
    pub kh_data1: KH_DATA1,
    #[doc = "0x18 - Lower Page Address"]
    pub page_addr0: PAGE_ADDR0,
    #[doc = "0x1c - Upper Page Address"]
    pub page_addr1: PAGE_ADDR1,
    #[doc = "0x20 - Key"]
    pub key: KEY,
    #[doc = "0x24 - Write Abort Address"]
    pub wr_abort_addr: WR_ABORT_ADDR,
    #[doc = "0x28 - Write Protection"]
    pub wrprot: WRPROT,
    #[doc = "0x2c - Signature"]
    pub signature: SIGNATURE,
    #[doc = "0x30 - User Configuration"]
    pub ucfg: UCFG,
    #[doc = "0x34 - Time Parameter 0"]
    pub time_param0: TIME_PARAM0,
    #[doc = "0x38 - Time Parameter 1"]
    pub time_param1: TIME_PARAM1,
    #[doc = "0x3c - IRQ Abort Enable (Lower Bits)"]
    pub abort_en_lo: ABORT_EN_LO,
    #[doc = "0x40 - IRQ Abort Enable (Upper Bits)"]
    pub abort_en_hi: ABORT_EN_HI,
    #[doc = "0x44 - ECC Configuration"]
    pub ecc_cfg: ECC_CFG,
    #[doc = "0x48 - ECC Status (Address)"]
    pub ecc_addr: ECC_ADDR,
    _reserved19: [u8; 4usize],
    #[doc = "0x50 - Flash Security"]
    pub por_sec: POR_SEC,
    #[doc = "0x54 - Volatile Flash Configuration"]
    pub vol_cfg: VOL_CFG,
}
#[doc = "Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u32, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "Status"]
pub mod stat;
#[doc = "Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien](ien) module"]
pub type IEN = crate::Reg<u32, _IEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN;
#[doc = "`read()` method returns [ien::R](ien::R) reader structure"]
impl crate::Readable for IEN {}
#[doc = "`write(|w| ..)` method takes [ien::W](ien::W) writer structure"]
impl crate::Writable for IEN {}
#[doc = "Interrupt Enable"]
pub mod ien;
#[doc = "Command\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cmd](cmd) module"]
pub type CMD = crate::Reg<u32, _CMD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CMD;
#[doc = "`read()` method returns [cmd::R](cmd::R) reader structure"]
impl crate::Readable for CMD {}
#[doc = "`write(|w| ..)` method takes [cmd::W](cmd::W) writer structure"]
impl crate::Writable for CMD {}
#[doc = "Command"]
pub mod cmd;
#[doc = "Write Address\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [kh_addr](kh_addr) module"]
pub type KH_ADDR = crate::Reg<u32, _KH_ADDR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _KH_ADDR;
#[doc = "`read()` method returns [kh_addr::R](kh_addr::R) reader structure"]
impl crate::Readable for KH_ADDR {}
#[doc = "`write(|w| ..)` method takes [kh_addr::W](kh_addr::W) writer structure"]
impl crate::Writable for KH_ADDR {}
#[doc = "Write Address"]
pub mod kh_addr;
#[doc = "Write Lower Data\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [kh_data0](kh_data0) module"]
pub type KH_DATA0 = crate::Reg<u32, _KH_DATA0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _KH_DATA0;
#[doc = "`read()` method returns [kh_data0::R](kh_data0::R) reader structure"]
impl crate::Readable for KH_DATA0 {}
#[doc = "`write(|w| ..)` method takes [kh_data0::W](kh_data0::W) writer structure"]
impl crate::Writable for KH_DATA0 {}
#[doc = "Write Lower Data"]
pub mod kh_data0;
#[doc = "Write Upper Data\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [kh_data1](kh_data1) module"]
pub type KH_DATA1 = crate::Reg<u32, _KH_DATA1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _KH_DATA1;
#[doc = "`read()` method returns [kh_data1::R](kh_data1::R) reader structure"]
impl crate::Readable for KH_DATA1 {}
#[doc = "`write(|w| ..)` method takes [kh_data1::W](kh_data1::W) writer structure"]
impl crate::Writable for KH_DATA1 {}
#[doc = "Write Upper Data"]
pub mod kh_data1;
#[doc = "Lower Page Address\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [page_addr0](page_addr0) module"]
pub type PAGE_ADDR0 = crate::Reg<u32, _PAGE_ADDR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PAGE_ADDR0;
#[doc = "`read()` method returns [page_addr0::R](page_addr0::R) reader structure"]
impl crate::Readable for PAGE_ADDR0 {}
#[doc = "`write(|w| ..)` method takes [page_addr0::W](page_addr0::W) writer structure"]
impl crate::Writable for PAGE_ADDR0 {}
#[doc = "Lower Page Address"]
pub mod page_addr0;
#[doc = "Upper Page Address\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [page_addr1](page_addr1) module"]
pub type PAGE_ADDR1 = crate::Reg<u32, _PAGE_ADDR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PAGE_ADDR1;
#[doc = "`read()` method returns [page_addr1::R](page_addr1::R) reader structure"]
impl crate::Readable for PAGE_ADDR1 {}
#[doc = "`write(|w| ..)` method takes [page_addr1::W](page_addr1::W) writer structure"]
impl crate::Writable for PAGE_ADDR1 {}
#[doc = "Upper Page Address"]
pub mod page_addr1;
#[doc = "Key\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [key](key) module"]
pub type KEY = crate::Reg<u32, _KEY>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _KEY;
#[doc = "`write(|w| ..)` method takes [key::W](key::W) writer structure"]
impl crate::Writable for KEY {}
#[doc = "Key"]
pub mod key;
#[doc = "Write Abort Address\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [wr_abort_addr](wr_abort_addr) module"]
pub type WR_ABORT_ADDR = crate::Reg<u32, _WR_ABORT_ADDR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _WR_ABORT_ADDR;
#[doc = "`read()` method returns [wr_abort_addr::R](wr_abort_addr::R) reader structure"]
impl crate::Readable for WR_ABORT_ADDR {}
#[doc = "Write Abort Address"]
pub mod wr_abort_addr;
#[doc = "Write Protection\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [wrprot](wrprot) module"]
pub type WRPROT = crate::Reg<u32, _WRPROT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _WRPROT;
#[doc = "`read()` method returns [wrprot::R](wrprot::R) reader structure"]
impl crate::Readable for WRPROT {}
#[doc = "`write(|w| ..)` method takes [wrprot::W](wrprot::W) writer structure"]
impl crate::Writable for WRPROT {}
#[doc = "Write Protection"]
pub mod wrprot;
#[doc = "Signature\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [signature](signature) module"]
pub type SIGNATURE = crate::Reg<u32, _SIGNATURE>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SIGNATURE;
#[doc = "`read()` method returns [signature::R](signature::R) reader structure"]
impl crate::Readable for SIGNATURE {}
#[doc = "Signature"]
pub mod signature;
#[doc = "User Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ucfg](ucfg) module"]
pub type UCFG = crate::Reg<u32, _UCFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _UCFG;
#[doc = "`read()` method returns [ucfg::R](ucfg::R) reader structure"]
impl crate::Readable for UCFG {}
#[doc = "`write(|w| ..)` method takes [ucfg::W](ucfg::W) writer structure"]
impl crate::Writable for UCFG {}
#[doc = "User Configuration"]
pub mod ucfg;
#[doc = "Time Parameter 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [time_param0](time_param0) module"]
pub type TIME_PARAM0 = crate::Reg<u32, _TIME_PARAM0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TIME_PARAM0;
#[doc = "`read()` method returns [time_param0::R](time_param0::R) reader structure"]
impl crate::Readable for TIME_PARAM0 {}
#[doc = "`write(|w| ..)` method takes [time_param0::W](time_param0::W) writer structure"]
impl crate::Writable for TIME_PARAM0 {}
#[doc = "Time Parameter 0"]
pub mod time_param0;
#[doc = "Time Parameter 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [time_param1](time_param1) module"]
pub type TIME_PARAM1 = crate::Reg<u32, _TIME_PARAM1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TIME_PARAM1;
#[doc = "`read()` method returns [time_param1::R](time_param1::R) reader structure"]
impl crate::Readable for TIME_PARAM1 {}
#[doc = "`write(|w| ..)` method takes [time_param1::W](time_param1::W) writer structure"]
impl crate::Writable for TIME_PARAM1 {}
#[doc = "Time Parameter 1"]
pub mod time_param1;
#[doc = "IRQ Abort Enable (Lower Bits)\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [abort_en_lo](abort_en_lo) module"]
pub type ABORT_EN_LO = crate::Reg<u32, _ABORT_EN_LO>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ABORT_EN_LO;
#[doc = "`read()` method returns [abort_en_lo::R](abort_en_lo::R) reader structure"]
impl crate::Readable for ABORT_EN_LO {}
#[doc = "`write(|w| ..)` method takes [abort_en_lo::W](abort_en_lo::W) writer structure"]
impl crate::Writable for ABORT_EN_LO {}
#[doc = "IRQ Abort Enable (Lower Bits)"]
pub mod abort_en_lo;
#[doc = "IRQ Abort Enable (Upper Bits)\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [abort_en_hi](abort_en_hi) module"]
pub type ABORT_EN_HI = crate::Reg<u32, _ABORT_EN_HI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ABORT_EN_HI;
#[doc = "`read()` method returns [abort_en_hi::R](abort_en_hi::R) reader structure"]
impl crate::Readable for ABORT_EN_HI {}
#[doc = "`write(|w| ..)` method takes [abort_en_hi::W](abort_en_hi::W) writer structure"]
impl crate::Writable for ABORT_EN_HI {}
#[doc = "IRQ Abort Enable (Upper Bits)"]
pub mod abort_en_hi;
#[doc = "ECC Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ecc_cfg](ecc_cfg) module"]
pub type ECC_CFG = crate::Reg<u32, _ECC_CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ECC_CFG;
#[doc = "`read()` method returns [ecc_cfg::R](ecc_cfg::R) reader structure"]
impl crate::Readable for ECC_CFG {}
#[doc = "`write(|w| ..)` method takes [ecc_cfg::W](ecc_cfg::W) writer structure"]
impl crate::Writable for ECC_CFG {}
#[doc = "ECC Configuration"]
pub mod ecc_cfg;
#[doc = "ECC Status (Address)\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ecc_addr](ecc_addr) module"]
pub type ECC_ADDR = crate::Reg<u32, _ECC_ADDR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ECC_ADDR;
#[doc = "`read()` method returns [ecc_addr::R](ecc_addr::R) reader structure"]
impl crate::Readable for ECC_ADDR {}
#[doc = "ECC Status (Address)"]
pub mod ecc_addr;
#[doc = "Flash Security\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [por_sec](por_sec) module"]
pub type POR_SEC = crate::Reg<u32, _POR_SEC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _POR_SEC;
#[doc = "`read()` method returns [por_sec::R](por_sec::R) reader structure"]
impl crate::Readable for POR_SEC {}
#[doc = "`write(|w| ..)` method takes [por_sec::W](por_sec::W) writer structure"]
impl crate::Writable for POR_SEC {}
#[doc = "Flash Security"]
pub mod por_sec;
#[doc = "Volatile Flash Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [vol_cfg](vol_cfg) module"]
pub type VOL_CFG = crate::Reg<u32, _VOL_CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _VOL_CFG;
#[doc = "`read()` method returns [vol_cfg::R](vol_cfg::R) reader structure"]
impl crate::Readable for VOL_CFG {}
#[doc = "`write(|w| ..)` method takes [vol_cfg::W](vol_cfg::W) writer structure"]
impl crate::Writable for VOL_CFG {}
#[doc = "Volatile Flash Configuration"]
pub mod vol_cfg;
