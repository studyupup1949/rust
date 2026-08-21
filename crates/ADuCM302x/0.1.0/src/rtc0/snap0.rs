#[doc = "Reader of register SNAP0"]
pub type R = crate::R<u16, super::SNAP0>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - Constituent Part of the 47-bit Input Capture Channel 0, Containing a Sticky Snapshot of CNT0"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xffff) as u16)
    }
}
