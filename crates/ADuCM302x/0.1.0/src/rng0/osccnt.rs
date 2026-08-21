#[doc = "Reader of register OSCCNT"]
pub type R = crate::R<u32, super::OSCCNT>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:27 - Oscillator Count"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x0fff_ffff) as u32)
    }
}
