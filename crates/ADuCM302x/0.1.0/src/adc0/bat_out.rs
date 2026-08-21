#[doc = "Reader of register BAT_OUT"]
pub type R = crate::R<u16, super::BAT_OUT>;
#[doc = "Reader of field `RESULT`"]
pub type RESULT_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - Conversion Result of Battery Monitoring"]
    #[inline(always)]
    pub fn result(&self) -> RESULT_R {
        RESULT_R::new((self.bits & 0xffff) as u16)
    }
}
