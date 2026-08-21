#[doc = "Writer for register IPBYTE"]
pub type W = crate::W<u8, super::IPBYTE>;
#[doc = "Register IPBYTE `reset()`'s with value 0"]
impl crate::ResetValue for super::IPBYTE {
    type Type = u8;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `DATA_BYTE`"]
pub struct DATA_BYTE_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA_BYTE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u8) & 0xff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:7 - Input Data Byte"]
    #[inline(always)]
    pub fn data_byte(&mut self) -> DATA_BYTE_W {
        DATA_BYTE_W { w: self }
    }
}
