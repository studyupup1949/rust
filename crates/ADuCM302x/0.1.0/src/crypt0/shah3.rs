#[doc = "Reader of register SHAH3"]
pub type R = crate::R<u32, super::SHAH3>;
#[doc = "Reader of field `SHAHASH3`"]
pub type SHAHASH3_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 3: SHA Hash"]
    #[inline(always)]
    pub fn shahash3(&self) -> SHAHASH3_R {
        SHAHASH3_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
