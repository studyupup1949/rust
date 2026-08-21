#[doc = "Reader of register OSCDIFF[%s]"]
pub type R = crate::R<u8, super::OSCDIFF>;
#[doc = "Reader of field `DELTA`"]
pub type DELTA_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - Oscillator Count Difference"]
    #[inline(always)]
    pub fn delta(&self) -> DELTA_R {
        DELTA_R::new((self.bits & 0xff) as u8)
    }
}
