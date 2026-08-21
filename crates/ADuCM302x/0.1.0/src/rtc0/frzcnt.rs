#[doc = "Reader of register FRZCNT"]
pub type R = crate::R<u16, super::FRZCNT>;
#[doc = "Reader of field `FRZCNT`"]
pub type FRZCNT_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - RTC Freeze Count. Coherent, Triple 16-Bit Read of the 47-Bit RTC Count"]
    #[inline(always)]
    pub fn frzcnt(&self) -> FRZCNT_R {
        FRZCNT_R::new((self.bits & 0xffff) as u16)
    }
}
