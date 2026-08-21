#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Configuration Register"]
    pub cfg: CFG,
    #[doc = "0x04 - Payload Data Length"]
    pub datalen: DATALEN,
    #[doc = "0x08 - Authentication Data Length"]
    pub prefixlen: PREFIXLEN,
    #[doc = "0x0c - Interrupt Enable Register"]
    pub inten: INTEN,
    #[doc = "0x10 - Status Register"]
    pub stat: STAT,
    #[doc = "0x14 - Input Buffer"]
    pub inbuf: INBUF,
    #[doc = "0x18 - Output Buffer"]
    pub outbuf: OUTBUF,
    #[doc = "0x1c - Nonce Bits \\[31:0\\]"]
    pub nonce0: NONCE0,
    #[doc = "0x20 - Nonce Bits \\[63:32\\]"]
    pub nonce1: NONCE1,
    #[doc = "0x24 - Nonce Bits \\[95:64\\]"]
    pub nonce2: NONCE2,
    #[doc = "0x28 - Nonce Bits \\[127:96\\]"]
    pub nonce3: NONCE3,
    #[doc = "0x2c - AES Key Bits \\[31:0\\]"]
    pub aeskey0: AESKEY0,
    #[doc = "0x30 - AES Key Bits \\[63:32\\]"]
    pub aeskey1: AESKEY1,
    #[doc = "0x34 - AES Key Bits \\[95:64\\]"]
    pub aeskey2: AESKEY2,
    #[doc = "0x38 - AES Key Bits \\[127:96\\]"]
    pub aeskey3: AESKEY3,
    #[doc = "0x3c - AES Key Bits \\[159:128\\]"]
    pub aeskey4: AESKEY4,
    #[doc = "0x40 - AES Key Bits \\[191:160\\]"]
    pub aeskey5: AESKEY5,
    #[doc = "0x44 - AES Key Bits \\[223:192\\]"]
    pub aeskey6: AESKEY6,
    #[doc = "0x48 - AES Key Bits \\[255:224\\]"]
    pub aeskey7: AESKEY7,
    #[doc = "0x4c - Counter Initialization Vector"]
    pub cntrinit: CNTRINIT,
    #[doc = "0x50 - SHA Bits \\[31:0\\]"]
    pub shah0: SHAH0,
    #[doc = "0x54 - SHA Bits \\[63:32\\]"]
    pub shah1: SHAH1,
    #[doc = "0x58 - SHA Bits \\[95:64\\]"]
    pub shah2: SHAH2,
    #[doc = "0x5c - SHA Bits \\[127:96\\]"]
    pub shah3: SHAH3,
    #[doc = "0x60 - SHA Bits \\[159:128\\]"]
    pub shah4: SHAH4,
    #[doc = "0x64 - SHA Bits \\[191:160\\]"]
    pub shah5: SHAH5,
    #[doc = "0x68 - SHA Bits \\[223:192\\]"]
    pub shah6: SHAH6,
    #[doc = "0x6c - SHA Bits \\[255:224\\]"]
    pub shah7: SHAH7,
    #[doc = "0x70 - SHA Last Word and Valid Bits Information"]
    pub sha_last_word: SHA_LAST_WORD,
    #[doc = "0x74 - NUM_VALID_BYTES"]
    pub ccm_num_valid_bytes: CCM_NUM_VALID_BYTES,
}
#[doc = "Configuration Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub type CFG = crate::Reg<u32, _CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG;
#[doc = "`read()` method returns [cfg::R](cfg::R) reader structure"]
impl crate::Readable for CFG {}
#[doc = "`write(|w| ..)` method takes [cfg::W](cfg::W) writer structure"]
impl crate::Writable for CFG {}
#[doc = "Configuration Register"]
pub mod cfg;
#[doc = "Payload Data Length\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [datalen](datalen) module"]
pub type DATALEN = crate::Reg<u32, _DATALEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DATALEN;
#[doc = "`read()` method returns [datalen::R](datalen::R) reader structure"]
impl crate::Readable for DATALEN {}
#[doc = "`write(|w| ..)` method takes [datalen::W](datalen::W) writer structure"]
impl crate::Writable for DATALEN {}
#[doc = "Payload Data Length"]
pub mod datalen;
#[doc = "Authentication Data Length\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [prefixlen](prefixlen) module"]
pub type PREFIXLEN = crate::Reg<u32, _PREFIXLEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PREFIXLEN;
#[doc = "`read()` method returns [prefixlen::R](prefixlen::R) reader structure"]
impl crate::Readable for PREFIXLEN {}
#[doc = "`write(|w| ..)` method takes [prefixlen::W](prefixlen::W) writer structure"]
impl crate::Writable for PREFIXLEN {}
#[doc = "Authentication Data Length"]
pub mod prefixlen;
#[doc = "Interrupt Enable Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [inten](inten) module"]
pub type INTEN = crate::Reg<u32, _INTEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTEN;
#[doc = "`read()` method returns [inten::R](inten::R) reader structure"]
impl crate::Readable for INTEN {}
#[doc = "`write(|w| ..)` method takes [inten::W](inten::W) writer structure"]
impl crate::Writable for INTEN {}
#[doc = "Interrupt Enable Register"]
pub mod inten;
#[doc = "Status Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u32, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "Status Register"]
pub mod stat;
#[doc = "Input Buffer\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [inbuf](inbuf) module"]
pub type INBUF = crate::Reg<u32, _INBUF>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INBUF;
#[doc = "`write(|w| ..)` method takes [inbuf::W](inbuf::W) writer structure"]
impl crate::Writable for INBUF {}
#[doc = "Input Buffer"]
pub mod inbuf;
#[doc = "Output Buffer\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [outbuf](outbuf) module"]
pub type OUTBUF = crate::Reg<u32, _OUTBUF>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OUTBUF;
#[doc = "`read()` method returns [outbuf::R](outbuf::R) reader structure"]
impl crate::Readable for OUTBUF {}
#[doc = "Output Buffer"]
pub mod outbuf;
#[doc = "Nonce Bits \\[31:0\\]\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [nonce0](nonce0) module"]
pub type NONCE0 = crate::Reg<u32, _NONCE0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NONCE0;
#[doc = "`read()` method returns [nonce0::R](nonce0::R) reader structure"]
impl crate::Readable for NONCE0 {}
#[doc = "`write(|w| ..)` method takes [nonce0::W](nonce0::W) writer structure"]
impl crate::Writable for NONCE0 {}
#[doc = "Nonce Bits \\[31:0\\]"]
pub mod nonce0;
#[doc = "Nonce Bits \\[63:32\\]\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [nonce1](nonce1) module"]
pub type NONCE1 = crate::Reg<u32, _NONCE1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NONCE1;
#[doc = "`read()` method returns [nonce1::R](nonce1::R) reader structure"]
impl crate::Readable for NONCE1 {}
#[doc = "`write(|w| ..)` method takes [nonce1::W](nonce1::W) writer structure"]
impl crate::Writable for NONCE1 {}
#[doc = "Nonce Bits \\[63:32\\]"]
pub mod nonce1;
#[doc = "Nonce Bits \\[95:64\\]\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [nonce2](nonce2) module"]
pub type NONCE2 = crate::Reg<u32, _NONCE2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NONCE2;
#[doc = "`read()` method returns [nonce2::R](nonce2::R) reader structure"]
impl crate::Readable for NONCE2 {}
#[doc = "`write(|w| ..)` method takes [nonce2::W](nonce2::W) writer structure"]
impl crate::Writable for NONCE2 {}
#[doc = "Nonce Bits \\[95:64\\]"]
pub mod nonce2;
#[doc = "Nonce Bits \\[127:96\\]\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [nonce3](nonce3) module"]
pub type NONCE3 = crate::Reg<u32, _NONCE3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _NONCE3;
#[doc = "`read()` method returns [nonce3::R](nonce3::R) reader structure"]
impl crate::Readable for NONCE3 {}
#[doc = "`write(|w| ..)` method takes [nonce3::W](nonce3::W) writer structure"]
impl crate::Writable for NONCE3 {}
#[doc = "Nonce Bits \\[127:96\\]"]
pub mod nonce3;
#[doc = "AES Key Bits \\[31:0\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey0](aeskey0) module"]
pub type AESKEY0 = crate::Reg<u32, _AESKEY0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY0;
#[doc = "`write(|w| ..)` method takes [aeskey0::W](aeskey0::W) writer structure"]
impl crate::Writable for AESKEY0 {}
#[doc = "AES Key Bits \\[31:0\\]"]
pub mod aeskey0;
#[doc = "AES Key Bits \\[63:32\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey1](aeskey1) module"]
pub type AESKEY1 = crate::Reg<u32, _AESKEY1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY1;
#[doc = "`write(|w| ..)` method takes [aeskey1::W](aeskey1::W) writer structure"]
impl crate::Writable for AESKEY1 {}
#[doc = "AES Key Bits \\[63:32\\]"]
pub mod aeskey1;
#[doc = "AES Key Bits \\[95:64\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey2](aeskey2) module"]
pub type AESKEY2 = crate::Reg<u32, _AESKEY2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY2;
#[doc = "`write(|w| ..)` method takes [aeskey2::W](aeskey2::W) writer structure"]
impl crate::Writable for AESKEY2 {}
#[doc = "AES Key Bits \\[95:64\\]"]
pub mod aeskey2;
#[doc = "AES Key Bits \\[127:96\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey3](aeskey3) module"]
pub type AESKEY3 = crate::Reg<u32, _AESKEY3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY3;
#[doc = "`write(|w| ..)` method takes [aeskey3::W](aeskey3::W) writer structure"]
impl crate::Writable for AESKEY3 {}
#[doc = "AES Key Bits \\[127:96\\]"]
pub mod aeskey3;
#[doc = "AES Key Bits \\[159:128\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey4](aeskey4) module"]
pub type AESKEY4 = crate::Reg<u32, _AESKEY4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY4;
#[doc = "`write(|w| ..)` method takes [aeskey4::W](aeskey4::W) writer structure"]
impl crate::Writable for AESKEY4 {}
#[doc = "AES Key Bits \\[159:128\\]"]
pub mod aeskey4;
#[doc = "AES Key Bits \\[191:160\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey5](aeskey5) module"]
pub type AESKEY5 = crate::Reg<u32, _AESKEY5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY5;
#[doc = "`write(|w| ..)` method takes [aeskey5::W](aeskey5::W) writer structure"]
impl crate::Writable for AESKEY5 {}
#[doc = "AES Key Bits \\[191:160\\]"]
pub mod aeskey5;
#[doc = "AES Key Bits \\[223:192\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey6](aeskey6) module"]
pub type AESKEY6 = crate::Reg<u32, _AESKEY6>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY6;
#[doc = "`write(|w| ..)` method takes [aeskey6::W](aeskey6::W) writer structure"]
impl crate::Writable for AESKEY6 {}
#[doc = "AES Key Bits \\[223:192\\]"]
pub mod aeskey6;
#[doc = "AES Key Bits \\[255:224\\]\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aeskey7](aeskey7) module"]
pub type AESKEY7 = crate::Reg<u32, _AESKEY7>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AESKEY7;
#[doc = "`write(|w| ..)` method takes [aeskey7::W](aeskey7::W) writer structure"]
impl crate::Writable for AESKEY7 {}
#[doc = "AES Key Bits \\[255:224\\]"]
pub mod aeskey7;
#[doc = "Counter Initialization Vector\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cntrinit](cntrinit) module"]
pub type CNTRINIT = crate::Reg<u32, _CNTRINIT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNTRINIT;
#[doc = "`read()` method returns [cntrinit::R](cntrinit::R) reader structure"]
impl crate::Readable for CNTRINIT {}
#[doc = "`write(|w| ..)` method takes [cntrinit::W](cntrinit::W) writer structure"]
impl crate::Writable for CNTRINIT {}
#[doc = "Counter Initialization Vector"]
pub mod cntrinit;
#[doc = "SHA Bits \\[31:0\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah0](shah0) module"]
pub type SHAH0 = crate::Reg<u32, _SHAH0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH0;
#[doc = "`read()` method returns [shah0::R](shah0::R) reader structure"]
impl crate::Readable for SHAH0 {}
#[doc = "SHA Bits \\[31:0\\]"]
pub mod shah0;
#[doc = "SHA Bits \\[63:32\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah1](shah1) module"]
pub type SHAH1 = crate::Reg<u32, _SHAH1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH1;
#[doc = "`read()` method returns [shah1::R](shah1::R) reader structure"]
impl crate::Readable for SHAH1 {}
#[doc = "SHA Bits \\[63:32\\]"]
pub mod shah1;
#[doc = "SHA Bits \\[95:64\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah2](shah2) module"]
pub type SHAH2 = crate::Reg<u32, _SHAH2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH2;
#[doc = "`read()` method returns [shah2::R](shah2::R) reader structure"]
impl crate::Readable for SHAH2 {}
#[doc = "SHA Bits \\[95:64\\]"]
pub mod shah2;
#[doc = "SHA Bits \\[127:96\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah3](shah3) module"]
pub type SHAH3 = crate::Reg<u32, _SHAH3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH3;
#[doc = "`read()` method returns [shah3::R](shah3::R) reader structure"]
impl crate::Readable for SHAH3 {}
#[doc = "SHA Bits \\[127:96\\]"]
pub mod shah3;
#[doc = "SHA Bits \\[159:128\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah4](shah4) module"]
pub type SHAH4 = crate::Reg<u32, _SHAH4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH4;
#[doc = "`read()` method returns [shah4::R](shah4::R) reader structure"]
impl crate::Readable for SHAH4 {}
#[doc = "SHA Bits \\[159:128\\]"]
pub mod shah4;
#[doc = "SHA Bits \\[191:160\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah5](shah5) module"]
pub type SHAH5 = crate::Reg<u32, _SHAH5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH5;
#[doc = "`read()` method returns [shah5::R](shah5::R) reader structure"]
impl crate::Readable for SHAH5 {}
#[doc = "SHA Bits \\[191:160\\]"]
pub mod shah5;
#[doc = "SHA Bits \\[223:192\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah6](shah6) module"]
pub type SHAH6 = crate::Reg<u32, _SHAH6>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH6;
#[doc = "`read()` method returns [shah6::R](shah6::R) reader structure"]
impl crate::Readable for SHAH6 {}
#[doc = "SHA Bits \\[223:192\\]"]
pub mod shah6;
#[doc = "SHA Bits \\[255:224\\]\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [shah7](shah7) module"]
pub type SHAH7 = crate::Reg<u32, _SHAH7>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHAH7;
#[doc = "`read()` method returns [shah7::R](shah7::R) reader structure"]
impl crate::Readable for SHAH7 {}
#[doc = "SHA Bits \\[255:224\\]"]
pub mod shah7;
#[doc = "SHA Last Word and Valid Bits Information\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sha_last_word](sha_last_word) module"]
pub type SHA_LAST_WORD = crate::Reg<u32, _SHA_LAST_WORD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SHA_LAST_WORD;
#[doc = "`read()` method returns [sha_last_word::R](sha_last_word::R) reader structure"]
impl crate::Readable for SHA_LAST_WORD {}
#[doc = "`write(|w| ..)` method takes [sha_last_word::W](sha_last_word::W) writer structure"]
impl crate::Writable for SHA_LAST_WORD {}
#[doc = "SHA Last Word and Valid Bits Information"]
pub mod sha_last_word;
#[doc = "NUM_VALID_BYTES\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ccm_num_valid_bytes](ccm_num_valid_bytes) module"]
pub type CCM_NUM_VALID_BYTES = crate::Reg<u32, _CCM_NUM_VALID_BYTES>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CCM_NUM_VALID_BYTES;
#[doc = "`read()` method returns [ccm_num_valid_bytes::R](ccm_num_valid_bytes::R) reader structure"]
impl crate::Readable for CCM_NUM_VALID_BYTES {}
#[doc = "`write(|w| ..)` method takes [ccm_num_valid_bytes::W](ccm_num_valid_bytes::W) writer structure"]
impl crate::Writable for CCM_NUM_VALID_BYTES {}
#[doc = "NUM_VALID_BYTES"]
pub mod ccm_num_valid_bytes;
