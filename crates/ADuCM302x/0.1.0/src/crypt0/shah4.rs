#[doc = "Reader of register SHAH4"]
pub type R = crate::R<u32, super::SHAH4>;
#[doc = "Reader of field `SHAHASH4`"]
pub type SHAHASH4_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Word 4: SHA Hash"]
    #[inline(always)]
    pub fn shahash4(&self) -> SHAHASH4_R {
        SHAHASH4_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
