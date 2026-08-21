#[doc = "Reader of register SHAH5"]
pub type R = crate::R<u32, super::SHAH5>;
#[doc = "Reader of field `SHAHASH5`"]
pub type SHAHASH5_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 5: SHA Hash"]
    #[inline(always)]
    pub fn shahash5(&self) -> SHAHASH5_R {
        SHAHASH5_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
