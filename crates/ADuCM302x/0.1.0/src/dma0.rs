#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - DMA Status"]
    pub stat: STAT,
    #[doc = "0x04 - DMA Configuration"]
    pub cfg: CFG,
    #[doc = "0x08 - DMA Channel Primary Control Database Pointer"]
    pub pdbptr: PDBPTR,
    #[doc = "0x0c - DMA Channel Alternate Control Database Pointer"]
    pub adbptr: ADBPTR,
    _reserved4: [u8; 4usize],
    #[doc = "0x14 - DMA Channel Software Request"]
    pub swreq: SWREQ,
    _reserved5: [u8; 8usize],
    #[doc = "0x20 - DMA Channel Request Mask Set"]
    pub rmsk_set: RMSK_SET,
    #[doc = "0x24 - DMA Channel Request Mask Clear"]
    pub rmsk_clr: RMSK_CLR,
    #[doc = "0x28 - DMA Channel Enable Set"]
    pub en_set: EN_SET,
    #[doc = "0x2c - DMA Channel Enable Clear"]
    pub en_clr: EN_CLR,
    #[doc = "0x30 - DMA Channel Primary Alternate Set"]
    pub alt_set: ALT_SET,
    #[doc = "0x34 - DMA Channel Primary Alternate Clear"]
    pub alt_clr: ALT_CLR,
    #[doc = "0x38 - DMA Channel Priority Set"]
    pub pri_set: PRI_SET,
    #[doc = "0x3c - DMA Channel Priority Clear"]
    pub pri_clr: PRI_CLR,
    _reserved13: [u8; 8usize],
    #[doc = "0x48 - DMA per Channel Error Clear"]
    pub errchnl_clr: ERRCHNL_CLR,
    #[doc = "0x4c - DMA Bus Error Clear"]
    pub err_clr: ERR_CLR,
    #[doc = "0x50 - DMA per Channel Invalid Descriptor Clear"]
    pub invaliddesc_clr: INVALIDDESC_CLR,
    _reserved16: [u8; 1964usize],
    #[doc = "0x800 - DMA Channel Bytes Swap Enable Set"]
    pub bs_set: BS_SET,
    #[doc = "0x804 - DMA Channel Bytes Swap Enable Clear"]
    pub bs_clr: BS_CLR,
    _reserved18: [u8; 8usize],
    #[doc = "0x810 - DMA Channel Source Address Decrement Enable Set"]
    pub srcaddr_set: SRCADDR_SET,
    #[doc = "0x814 - DMA Channel Source Address Decrement Enable Clear"]
    pub srcaddr_clr: SRCADDR_CLR,
    #[doc = "0x818 - DMA Channel Destination Address Decrement Enable Set"]
    pub dstaddr_set: DSTADDR_SET,
    #[doc = "0x81c - DMA Channel Destination Address Decrement Enable Clear"]
    pub dstaddr_clr: DSTADDR_CLR,
    _reserved22: [u8; 1984usize],
    #[doc = "0xfe0 - DMA Controller Revision ID"]
    pub revid: REVID,
}
#[doc = "DMA Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u32, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "DMA Status"]
pub mod stat;
#[doc = "DMA Configuration\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub type CFG = crate::Reg<u32, _CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG;
#[doc = "`write(|w| ..)` method takes [cfg::W](cfg::W) writer structure"]
impl crate::Writable for CFG {}
#[doc = "DMA Configuration"]
pub mod cfg;
#[doc = "DMA Channel Primary Control Database Pointer\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pdbptr](pdbptr) module"]
pub type PDBPTR = crate::Reg<u32, _PDBPTR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PDBPTR;
#[doc = "`read()` method returns [pdbptr::R](pdbptr::R) reader structure"]
impl crate::Readable for PDBPTR {}
#[doc = "`write(|w| ..)` method takes [pdbptr::W](pdbptr::W) writer structure"]
impl crate::Writable for PDBPTR {}
#[doc = "DMA Channel Primary Control Database Pointer"]
pub mod pdbptr;
#[doc = "DMA Channel Alternate Control Database Pointer\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [adbptr](adbptr) module"]
pub type ADBPTR = crate::Reg<u32, _ADBPTR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ADBPTR;
#[doc = "`read()` method returns [adbptr::R](adbptr::R) reader structure"]
impl crate::Readable for ADBPTR {}
#[doc = "DMA Channel Alternate Control Database Pointer"]
pub mod adbptr;
#[doc = "DMA Channel Software Request\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [swreq](swreq) module"]
pub type SWREQ = crate::Reg<u32, _SWREQ>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SWREQ;
#[doc = "`write(|w| ..)` method takes [swreq::W](swreq::W) writer structure"]
impl crate::Writable for SWREQ {}
#[doc = "DMA Channel Software Request"]
pub mod swreq;
#[doc = "DMA Channel Request Mask Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rmsk_set](rmsk_set) module"]
pub type RMSK_SET = crate::Reg<u32, _RMSK_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RMSK_SET;
#[doc = "`read()` method returns [rmsk_set::R](rmsk_set::R) reader structure"]
impl crate::Readable for RMSK_SET {}
#[doc = "`write(|w| ..)` method takes [rmsk_set::W](rmsk_set::W) writer structure"]
impl crate::Writable for RMSK_SET {}
#[doc = "DMA Channel Request Mask Set"]
pub mod rmsk_set;
#[doc = "DMA Channel Request Mask Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rmsk_clr](rmsk_clr) module"]
pub type RMSK_CLR = crate::Reg<u32, _RMSK_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RMSK_CLR;
#[doc = "`write(|w| ..)` method takes [rmsk_clr::W](rmsk_clr::W) writer structure"]
impl crate::Writable for RMSK_CLR {}
#[doc = "DMA Channel Request Mask Clear"]
pub mod rmsk_clr;
#[doc = "DMA Channel Enable Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [en_set](en_set) module"]
pub type EN_SET = crate::Reg<u32, _EN_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _EN_SET;
#[doc = "`read()` method returns [en_set::R](en_set::R) reader structure"]
impl crate::Readable for EN_SET {}
#[doc = "`write(|w| ..)` method takes [en_set::W](en_set::W) writer structure"]
impl crate::Writable for EN_SET {}
#[doc = "DMA Channel Enable Set"]
pub mod en_set;
#[doc = "DMA Channel Enable Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [en_clr](en_clr) module"]
pub type EN_CLR = crate::Reg<u32, _EN_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _EN_CLR;
#[doc = "`write(|w| ..)` method takes [en_clr::W](en_clr::W) writer structure"]
impl crate::Writable for EN_CLR {}
#[doc = "DMA Channel Enable Clear"]
pub mod en_clr;
#[doc = "DMA Channel Primary Alternate Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alt_set](alt_set) module"]
pub type ALT_SET = crate::Reg<u32, _ALT_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALT_SET;
#[doc = "`read()` method returns [alt_set::R](alt_set::R) reader structure"]
impl crate::Readable for ALT_SET {}
#[doc = "`write(|w| ..)` method takes [alt_set::W](alt_set::W) writer structure"]
impl crate::Writable for ALT_SET {}
#[doc = "DMA Channel Primary Alternate Set"]
pub mod alt_set;
#[doc = "DMA Channel Primary Alternate Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alt_clr](alt_clr) module"]
pub type ALT_CLR = crate::Reg<u32, _ALT_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALT_CLR;
#[doc = "`write(|w| ..)` method takes [alt_clr::W](alt_clr::W) writer structure"]
impl crate::Writable for ALT_CLR {}
#[doc = "DMA Channel Primary Alternate Clear"]
pub mod alt_clr;
#[doc = "DMA Channel Priority Set\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pri_set](pri_set) module"]
pub type PRI_SET = crate::Reg<u32, _PRI_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PRI_SET;
#[doc = "`write(|w| ..)` method takes [pri_set::W](pri_set::W) writer structure"]
impl crate::Writable for PRI_SET {}
#[doc = "DMA Channel Priority Set"]
pub mod pri_set;
#[doc = "DMA Channel Priority Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pri_clr](pri_clr) module"]
pub type PRI_CLR = crate::Reg<u32, _PRI_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PRI_CLR;
#[doc = "`write(|w| ..)` method takes [pri_clr::W](pri_clr::W) writer structure"]
impl crate::Writable for PRI_CLR {}
#[doc = "DMA Channel Priority Clear"]
pub mod pri_clr;
#[doc = "DMA per Channel Error Clear\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [errchnl_clr](errchnl_clr) module"]
pub type ERRCHNL_CLR = crate::Reg<u32, _ERRCHNL_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ERRCHNL_CLR;
#[doc = "`read()` method returns [errchnl_clr::R](errchnl_clr::R) reader structure"]
impl crate::Readable for ERRCHNL_CLR {}
#[doc = "`write(|w| ..)` method takes [errchnl_clr::W](errchnl_clr::W) writer structure"]
impl crate::Writable for ERRCHNL_CLR {}
#[doc = "DMA per Channel Error Clear"]
pub mod errchnl_clr;
#[doc = "DMA Bus Error Clear\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [err_clr](err_clr) module"]
pub type ERR_CLR = crate::Reg<u32, _ERR_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ERR_CLR;
#[doc = "`read()` method returns [err_clr::R](err_clr::R) reader structure"]
impl crate::Readable for ERR_CLR {}
#[doc = "`write(|w| ..)` method takes [err_clr::W](err_clr::W) writer structure"]
impl crate::Writable for ERR_CLR {}
#[doc = "DMA Bus Error Clear"]
pub mod err_clr;
#[doc = "DMA per Channel Invalid Descriptor Clear\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [invaliddesc_clr](invaliddesc_clr) module"]
pub type INVALIDDESC_CLR = crate::Reg<u32, _INVALIDDESC_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INVALIDDESC_CLR;
#[doc = "`read()` method returns [invaliddesc_clr::R](invaliddesc_clr::R) reader structure"]
impl crate::Readable for INVALIDDESC_CLR {}
#[doc = "`write(|w| ..)` method takes [invaliddesc_clr::W](invaliddesc_clr::W) writer structure"]
impl crate::Writable for INVALIDDESC_CLR {}
#[doc = "DMA per Channel Invalid Descriptor Clear"]
pub mod invaliddesc_clr;
#[doc = "DMA Channel Bytes Swap Enable Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [bs_set](bs_set) module"]
pub type BS_SET = crate::Reg<u32, _BS_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _BS_SET;
#[doc = "`read()` method returns [bs_set::R](bs_set::R) reader structure"]
impl crate::Readable for BS_SET {}
#[doc = "`write(|w| ..)` method takes [bs_set::W](bs_set::W) writer structure"]
impl crate::Writable for BS_SET {}
#[doc = "DMA Channel Bytes Swap Enable Set"]
pub mod bs_set;
#[doc = "DMA Channel Bytes Swap Enable Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [bs_clr](bs_clr) module"]
pub type BS_CLR = crate::Reg<u32, _BS_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _BS_CLR;
#[doc = "`write(|w| ..)` method takes [bs_clr::W](bs_clr::W) writer structure"]
impl crate::Writable for BS_CLR {}
#[doc = "DMA Channel Bytes Swap Enable Clear"]
pub mod bs_clr;
#[doc = "DMA Channel Source Address Decrement Enable Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [srcaddr_set](srcaddr_set) module"]
pub type SRCADDR_SET = crate::Reg<u32, _SRCADDR_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRCADDR_SET;
#[doc = "`read()` method returns [srcaddr_set::R](srcaddr_set::R) reader structure"]
impl crate::Readable for SRCADDR_SET {}
#[doc = "`write(|w| ..)` method takes [srcaddr_set::W](srcaddr_set::W) writer structure"]
impl crate::Writable for SRCADDR_SET {}
#[doc = "DMA Channel Source Address Decrement Enable Set"]
pub mod srcaddr_set;
#[doc = "DMA Channel Source Address Decrement Enable Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [srcaddr_clr](srcaddr_clr) module"]
pub type SRCADDR_CLR = crate::Reg<u32, _SRCADDR_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SRCADDR_CLR;
#[doc = "`write(|w| ..)` method takes [srcaddr_clr::W](srcaddr_clr::W) writer structure"]
impl crate::Writable for SRCADDR_CLR {}
#[doc = "DMA Channel Source Address Decrement Enable Clear"]
pub mod srcaddr_clr;
#[doc = "DMA Channel Destination Address Decrement Enable Set\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dstaddr_set](dstaddr_set) module"]
pub type DSTADDR_SET = crate::Reg<u32, _DSTADDR_SET>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DSTADDR_SET;
#[doc = "`read()` method returns [dstaddr_set::R](dstaddr_set::R) reader structure"]
impl crate::Readable for DSTADDR_SET {}
#[doc = "`write(|w| ..)` method takes [dstaddr_set::W](dstaddr_set::W) writer structure"]
impl crate::Writable for DSTADDR_SET {}
#[doc = "DMA Channel Destination Address Decrement Enable Set"]
pub mod dstaddr_set;
#[doc = "DMA Channel Destination Address Decrement Enable Clear\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dstaddr_clr](dstaddr_clr) module"]
pub type DSTADDR_CLR = crate::Reg<u32, _DSTADDR_CLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DSTADDR_CLR;
#[doc = "`write(|w| ..)` method takes [dstaddr_clr::W](dstaddr_clr::W) writer structure"]
impl crate::Writable for DSTADDR_CLR {}
#[doc = "DMA Channel Destination Address Decrement Enable Clear"]
pub mod dstaddr_clr;
#[doc = "DMA Controller Revision ID\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [revid](revid) module"]
pub type REVID = crate::Reg<u32, _REVID>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _REVID;
#[doc = "`read()` method returns [revid::R](revid::R) reader structure"]
impl crate::Readable for REVID {}
#[doc = "DMA Controller Revision ID"]
pub mod revid;
