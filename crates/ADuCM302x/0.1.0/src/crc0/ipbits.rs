#[doc = "Writer for register IPBITS[%s]"]
pub type W = crate::W<u8, super::IPBITS>;
#[doc = "Register IPBITS[%s]
`reset()`'s with value 0"]
impl crate::ResetValue for super::IPBITS {
    type Type = u8;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `DATA_BITS`"]
pub struct DATA_BITS_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA_BITS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u8) & 0xff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:7 - Input Data Bits"]
    #[inline(always)]
    pub fn data_bits(&mut self) -> DATA_BITS_W {
        DATA_BITS_W { w: self }
    }
}
