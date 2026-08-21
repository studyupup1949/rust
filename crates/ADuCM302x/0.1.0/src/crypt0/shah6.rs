#[doc = "Reader of register SHAH6"]
pub type R = crate::R<u32, super::SHAH6>;
#[doc = "Reader of field `SHAHASH6`"]
pub type SHAHASH6_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 6: SHA Hash"]
    #[inline(always)]
    pub fn shahash6(&self) -> SHAHASH6_R {
        SHAHASH6_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
