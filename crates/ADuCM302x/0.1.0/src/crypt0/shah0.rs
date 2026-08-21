#[doc = "Reader of register SHAH0"]
pub type R = crate::R<u32, super::SHAH0>;
#[doc = "Reader of field `SHAHASH0`"]
pub type SHAHASH0_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 0: SHA Hash"]
    #[inline(always)]
    pub fn shahash0(&self) -> SHAHASH0_R {
        SHAHASH0_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
