#[doc = "Reader of register HYS3"]
pub type R = crate::R<u16, super::HYS3>;
#[doc = "Writer for register HYS3"]
pub type W = crate::W<u16, super::HYS3>;
#[doc = "Register HYS3 `reset()`'s with value 0"]
impl crate::ResetValue for super::HYS3 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01ff) | ((value as u16) & 0x01ff);
        self.w
    }
}
#[doc = "Reader of field `MONCYC`"]
pub type MONCYC_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `MONCYC`"]
pub struct MONCYC_W<'a> {
    w: &'a mut W,
}
impl<'a> MONCYC_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 12)) | (((value as u16) & 0x07) << 12);
        self.w
    }
}
#[doc = "Reader of field `EN`"]
pub type EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EN`"]
pub struct EN_W<'a> {
    w: &'a mut W,
}
impl<'a> EN_W<'a> {
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
    #[doc = "Bits 0:8 - Hysteresis Value for Channel 3"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x01ff) as u16)
    }
    #[doc = "Bits 12:14 - Number of Conversion Cycles to Monitor Channel 3"]
    #[inline(always)]
    pub fn moncyc(&self) -> MONCYC_R {
        MONCYC_R::new(((self.bits >> 12) & 0x07) as u8)
    }
    #[doc = "Bit 15 - Enable Hysteresis for Comparison on Channel 3"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:8 - Hysteresis Value for Channel 3"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
    #[doc = "Bits 12:14 - Number of Conversion Cycles to Monitor Channel 3"]
    #[inline(always)]
    pub fn moncyc(&mut self) -> MONCYC_W {
        MONCYC_W { w: self }
    }
    #[doc = "Bit 15 - Enable Hysteresis for Comparison on Channel 3"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
}
