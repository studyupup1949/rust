#[doc = "Reader of register SHAH1"]
pub type R = crate::R<u32, super::SHAH1>;
#[doc = "Reader of field `SHAHASH1`"]
pub type SHAHASH1_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 1: SHA Hash"]
    #[inline(always)]
    pub fn shahash1(&self) -> SHAHASH1_R {
        SHAHASH1_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
