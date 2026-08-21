#[doc = "Reader of register CNV_CFG"]
pub type R = crate::R<u16, super::CNV_CFG>;
#[doc = "Writer for register CNV_CFG"]
pub type W = crate::W<u16, super::CNV_CFG>;
#[doc = "Register CNV_CFG `reset()`'s with value 0"]
impl crate::ResetValue for super::CNV_CFG {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SEL`"]
pub type SEL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SEL`"]
pub struct SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> SEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `BAT`"]
pub type BAT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BAT`"]
pub struct BAT_W<'a> {
    w: &'a mut W,
}
impl<'a> BAT_W<'a> {
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
#[doc = "Reader of field `TMP`"]
pub type TMP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TMP`"]
pub struct TMP_W<'a> {
    w: &'a mut W,
}
impl<'a> TMP_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u16) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `TMP2`"]
pub type TMP2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TMP2`"]
pub struct TMP2_W<'a> {
    w: &'a mut W,
}
impl<'a> TMP2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u16) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `AUTOMODE`"]
pub type AUTOMODE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AUTOMODE`"]
pub struct AUTOMODE_W<'a> {
    w: &'a mut W,
}
impl<'a> AUTOMODE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u16) & 0x01) << 12);
        self.w
    }
}
#[doc = "Reader of field `DMAEN`"]
pub type DMAEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DMAEN`"]
pub struct DMAEN_W<'a> {
    w: &'a mut W,
}
impl<'a> DMAEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u16) & 0x01) << 13);
        self.w
    }
}
#[doc = "Reader of field `SINGLE`"]
pub type SINGLE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SINGLE`"]
pub struct SINGLE_W<'a> {
    w: &'a mut W,
}
impl<'a> SINGLE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u16) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `MULTI`"]
pub type MULTI_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MULTI`"]
pub struct MULTI_W<'a> {
    w: &'a mut W,
}
impl<'a> MULTI_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u16) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Selection of Channel(s) to Convert"]
    #[inline(always)]
    pub fn sel(&self) -> SEL_R {
        SEL_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 8 - Battery Monitoring Enable"]
    #[inline(always)]
    pub fn bat(&self) -> BAT_R {
        BAT_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Temperature Measurement 1"]
    #[inline(always)]
    pub fn tmp(&self) -> TMP_R {
        TMP_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Temperature Measurement 2"]
    #[inline(always)]
    pub fn tmp2(&self) -> TMP2_R {
        TMP2_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Auto Mode Enable"]
    #[inline(always)]
    pub fn automode(&self) -> AUTOMODE_R {
        AUTOMODE_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - DMA Channel Enable"]
    #[inline(always)]
    pub fn dmaen(&self) -> DMAEN_R {
        DMAEN_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Single Conversion Start"]
    #[inline(always)]
    pub fn single(&self) -> SINGLE_R {
        SINGLE_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Multiple Conversions"]
    #[inline(always)]
    pub fn multi(&self) -> MULTI_R {
        MULTI_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Selection of Channel(s) to Convert"]
    #[inline(always)]
    pub fn sel(&mut self) -> SEL_W {
        SEL_W { w: self }
    }
    #[doc = "Bit 8 - Battery Monitoring Enable"]
    #[inline(always)]
    pub fn bat(&mut self) -> BAT_W {
        BAT_W { w: self }
    }
    #[doc = "Bit 9 - Temperature Measurement 1"]
    #[inline(always)]
    pub fn tmp(&mut self) -> TMP_W {
        TMP_W { w: self }
    }
    #[doc = "Bit 10 - Temperature Measurement 2"]
    #[inline(always)]
    pub fn tmp2(&mut self) -> TMP2_W {
        TMP2_W { w: self }
    }
    #[doc = "Bit 12 - Auto Mode Enable"]
    #[inline(always)]
    pub fn automode(&mut self) -> AUTOMODE_W {
        AUTOMODE_W { w: self }
    }
    #[doc = "Bit 13 - DMA Channel Enable"]
    #[inline(always)]
    pub fn dmaen(&mut self) -> DMAEN_W {
        DMAEN_W { w: self }
    }
    #[doc = "Bit 14 - Single Conversion Start"]
    #[inline(always)]
    pub fn single(&mut self) -> SINGLE_W {
        SINGLE_W { w: self }
    }
    #[doc = "Bit 15 - Multiple Conversions"]
    #[inline(always)]
    pub fn multi(&mut self) -> MULTI_W {
        MULTI_W { w: self }
    }
}
