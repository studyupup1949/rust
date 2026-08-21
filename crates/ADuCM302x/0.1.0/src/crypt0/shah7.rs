#[doc = "Reader of register SHAH7"]
pub type R = crate::R<u32, super::SHAH7>;
#[doc = "Reader of field `SHAHASH7`"]
pub type SHAHASH7_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 7: SHA Hash"]
    #[inline(always)]
    pub fn shahash7(&self) -> SHAHASH7_R {
        SHAHASH7_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
