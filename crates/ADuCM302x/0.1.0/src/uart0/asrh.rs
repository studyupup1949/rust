#[doc = "Reader of register ASRH"]
pub type R = crate::R<u16, super::ASRH>;
#[doc = "Reader of field `CNT`"]
pub type CNT_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - CNT\\[19:12\\]
Auto Baud Counter Value"]
    #[inline(always)]
    pub fn cnt(&self) -> CNT_R {
        CNT_R::new((self.bits & 0xff) as u8)
    }
}
