#[doc = "Reader of register SHAH2"]
pub type R = crate::R<u32, super::SHAH2>;
#[doc = "Reader of field `SHAHASH2`"]
pub type SHAHASH2_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 2: SHA Hash"]
    #[inline(always)]
    pub fn shahash2(&self) -> SHAHASH2_R {
        SHAHASH2_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
