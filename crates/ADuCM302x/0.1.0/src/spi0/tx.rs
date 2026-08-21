#[doc = "Writer for register TX"]
pub type W = crate::W<u16, super::TX>;
#[doc = "Register TX `reset()`'s with value 0"]
impl crate::ResetValue for super::TX {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `BYTE1`"]
pub struct BYTE1_W<'a> {
    w: &'a mut W,
}
impl<'a> BYTE1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Write proxy for field `BYTE2`"]
pub struct BYTE2_W<'a> {
    w: &'a mut W,
}
impl<'a> BYTE2_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 8)) | (((value as u16) & 0xff) << 8);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:7 - 8-bit Transmit Buffer"]
    #[inline(always)]
    pub fn byte1(&mut self) -> BYTE1_W {
        BYTE1_W { w: self }
    }
    #[doc = "Bits 8:15 - 8-bit Transmit Buffer, Used Only in DMA Modes"]
    #[inline(always)]
    pub fn byte2(&mut self) -> BYTE2_W {
        BYTE2_W { w: self }
    }
}
