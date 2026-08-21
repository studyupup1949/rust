#[doc = "Reader of register RFC"]
pub type R = crate::R<u16, super::RFC>;
#[doc = "Reader of field `RFC`"]
pub type RFC_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:4 - Current Rx FIFO Data Bytes"]
    #[inline(always)]
    pub fn rfc(&self) -> RFC_R {
        RFC_R::new((self.bits & 0x1f) as u8)
    }
}
