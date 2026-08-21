#[doc = "Reader of register STAT"]
pub type R = crate::R<u32, super::STAT>;
#[doc = "Reader of field `ICEN`"]
pub type ICEN_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - I-Cache Enabled"]
    #[inline(always)]
    pub fn icen(&self) -> ICEN_R {
        ICEN_R::new((self.bits & 0x01) != 0)
    }
}
