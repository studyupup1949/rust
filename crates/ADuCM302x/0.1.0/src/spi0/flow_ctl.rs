#[doc = "Reader of register FLOW_CTL"]
pub type R = crate::R<u16, super::FLOW_CTL>;
#[doc = "Writer for register FLOW_CTL"]
pub type W = crate::W<u16, super::FLOW_CTL>;
#[doc = "Register FLOW_CTL `reset()`'s with value 0"]
impl crate::ResetValue for super::FLOW_CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `MODE`"]
pub type MODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `MODE`"]
pub struct MODE_W<'a> {
    w: &'a mut W,
}
impl<'a> MODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u16) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `RDYPOL`"]
pub type RDYPOL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RDYPOL`"]
pub struct RDYPOL_W<'a> {
    w: &'a mut W,
}
impl<'a> RDYPOL_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `RDBURSTSZ`"]
pub type RDBURSTSZ_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `RDBURSTSZ`"]
pub struct RDBURSTSZ_W<'a> {
    w: &'a mut W,
}
impl<'a> RDBURSTSZ_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 8)) | (((value as u16) & 0x0f) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Flow Control Mode"]
    #[inline(always)]
    pub fn mode(&self) -> MODE_R {
        MODE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bit 4 - Polarity of RDY/MISO Line"]
    #[inline(always)]
    pub fn rdypol(&self) -> RDYPOL_R {
        RDYPOL_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 8:11 - Read Data Burst Size - 1"]
    #[inline(always)]
    pub fn rdburstsz(&self) -> RDBURSTSZ_R {
        RDBURSTSZ_R::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Flow Control Mode"]
    #[inline(always)]
    pub fn mode(&mut self) -> MODE_W {
        MODE_W { w: self }
    }
    #[doc = "Bit 4 - Polarity of RDY/MISO Line"]
    #[inline(always)]
    pub fn rdypol(&mut self) -> RDYPOL_W {
        RDYPOL_W { w: self }
    }
    #[doc = "Bits 8:11 - Read Data Burst Size - 1"]
    #[inline(always)]
    pub fn rdburstsz(&mut self) -> RDBURSTSZ_W {
        RDBURSTSZ_W { w: self }
    }
}
