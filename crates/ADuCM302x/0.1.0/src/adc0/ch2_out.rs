#[doc = "Reader of register CH2_OUT"]
pub type R = crate::R<u16, super::CH2_OUT>;
#[doc = "Reader of field `RESULT`"]
pub type RESULT_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - Conversion Result of Channel 2"]
    #[inline(always)]
    pub fn result(&self) -> RESULT_R {
        RESULT_R::new((self.bits & 0xffff) as u16)
    }
}
