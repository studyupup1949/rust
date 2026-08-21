#[doc = "Reader of register RD_CTL"]
pub type R = crate::R<u16, super::RD_CTL>;
#[doc = "Writer for register RD_CTL"]
pub type W = crate::W<u16, super::RD_CTL>;
#[doc = "Register RD_CTL `reset()`'s with value 0"]
impl crate::ResetValue for super::RD_CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CMDEN`"]
pub type CMDEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CMDEN`"]
pub struct CMDEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CMDEN_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u16) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `OVERLAP`"]
pub type OVERLAP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OVERLAP`"]
pub struct OVERLAP_W<'a> {
    w: &'a mut W,
}
impl<'a> OVERLAP_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u16) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `TXBYTES`"]
pub type TXBYTES_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TXBYTES`"]
pub struct TXBYTES_W<'a> {
    w: &'a mut W,
}
impl<'a> TXBYTES_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 2)) | (((value as u16) & 0x0f) << 2);
        self.w
    }
}
#[doc = "Reader of field `THREEPIN`"]
pub type THREEPIN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `THREEPIN`"]
pub struct THREEPIN_W<'a> {
    w: &'a mut W,
}
impl<'a> THREEPIN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Read Command Enable"]
    #[inline(always)]
    pub fn cmden(&self) -> CMDEN_R {
        CMDEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Tx/Rx Overlap Mode"]
    #[inline(always)]
    pub fn overlap(&self) -> OVERLAP_R {
        OVERLAP_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bits 2:5 - Transmit Byte Count - 1 (Read Command)"]
    #[inline(always)]
    pub fn txbytes(&self) -> TXBYTES_R {
        TXBYTES_R::new(((self.bits >> 2) & 0x0f) as u8)
    }
    #[doc = "Bit 8 - Three Pin SPI Mode"]
    #[inline(always)]
    pub fn threepin(&self) -> THREEPIN_R {
        THREEPIN_R::new(((self.bits >> 8) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Read Command Enable"]
    #[inline(always)]
    pub fn cmden(&mut self) -> CMDEN_W {
        CMDEN_W { w: self }
    }
    #[doc = "Bit 1 - Tx/Rx Overlap Mode"]
    #[inline(always)]
    pub fn overlap(&mut self) -> OVERLAP_W {
        OVERLAP_W { w: self }
    }
    #[doc = "Bits 2:5 - Transmit Byte Count - 1 (Read Command)"]
    #[inline(always)]
    pub fn txbytes(&mut self) -> TXBYTES_W {
        TXBYTES_W { w: self }
    }
    #[doc = "Bit 8 - Three Pin SPI Mode"]
    #[inline(always)]
    pub fn threepin(&mut self) -> THREEPIN_W {
        THREEPIN_W { w: self }
    }
}
