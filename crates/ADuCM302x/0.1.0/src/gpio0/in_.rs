#[doc = "Reader of register IN"]
pub type R = crate::R<u16, super::IN>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - Registered Data Input"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xffff) as u16)
    }
}
