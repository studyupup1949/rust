#[doc = "Reader of register STAT0"]
pub type R = crate::R<u32, super::STAT0>;
#[doc = "Writer for register STAT0"]
pub type W = crate::W<u32, super::STAT0>;
#[doc = "Register STAT0 `reset()`'s with value 0"]
impl crate::ResetValue for super::STAT0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SPLL`"]
pub type SPLL_R = crate::R<bool, bool>;
#[doc = "Reader of field `SPLLLK`"]
pub type SPLLLK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLLK`"]
pub struct SPLLLK_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLLK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `SPLLUNLK`"]
pub type SPLLUNLK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLUNLK`"]
pub struct SPLLUNLK_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLUNLK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `LFXTAL`"]
pub type LFXTAL_R = crate::R<bool, bool>;
#[doc = "Reader of field `LFXTALOK`"]
pub type LFXTALOK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTALOK`"]
pub struct LFXTALOK_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTALOK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u32) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `LFXTALNOK`"]
pub type LFXTALNOK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTALNOK`"]
pub struct LFXTALNOK_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTALNOK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `HFXTAL`"]
pub type HFXTAL_R = crate::R<bool, bool>;
#[doc = "Reader of field `HFXTALOK`"]
pub type HFXTALOK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HFXTALOK`"]
pub struct HFXTALOK_W<'a> {
    w: &'a mut W,
}
impl<'a> HFXTALOK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u32) & 0x01) << 13);
        self.w
    }
}
#[doc = "Reader of field `HFXTALNOK`"]
pub type HFXTALNOK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HFXTALNOK`"]
pub struct HFXTALNOK_W<'a> {
    w: &'a mut W,
}
impl<'a> HFXTALNOK_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u32) & 0x01) << 14);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - System PLL Status"]
    #[inline(always)]
    pub fn spll(&self) -> SPLL_R {
        SPLL_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - System PLL Lock"]
    #[inline(always)]
    pub fn splllk(&self) -> SPLLLK_R {
        SPLLLK_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - System PLL Unlock"]
    #[inline(always)]
    pub fn spllunlk(&self) -> SPLLUNLK_R {
        SPLLUNLK_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 8 - LF Crystal Status"]
    #[inline(always)]
    pub fn lfxtal(&self) -> LFXTAL_R {
        LFXTAL_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - LF Crystal Stable"]
    #[inline(always)]
    pub fn lfxtalok(&self) -> LFXTALOK_R {
        LFXTALOK_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - LF Crystal Not Stable"]
    #[inline(always)]
    pub fn lfxtalnok(&self) -> LFXTALNOK_R {
        LFXTALNOK_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 12 - HF Crystal Status"]
    #[inline(always)]
    pub fn hfxtal(&self) -> HFXTAL_R {
        HFXTAL_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - HF Crystal Stable"]
    #[inline(always)]
    pub fn hfxtalok(&self) -> HFXTALOK_R {
        HFXTALOK_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - HF Crystal Not Stable"]
    #[inline(always)]
    pub fn hfxtalnok(&self) -> HFXTALNOK_R {
        HFXTALNOK_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - System PLL Lock"]
    #[inline(always)]
    pub fn splllk(&mut self) -> SPLLLK_W {
        SPLLLK_W { w: self }
    }
    #[doc = "Bit 2 - System PLL Unlock"]
    #[inline(always)]
    pub fn spllunlk(&mut self) -> SPLLUNLK_W {
        SPLLUNLK_W { w: self }
    }
    #[doc = "Bit 9 - LF Crystal Stable"]
    #[inline(always)]
    pub fn lfxtalok(&mut self) -> LFXTALOK_W {
        LFXTALOK_W { w: self }
    }
    #[doc = "Bit 10 - LF Crystal Not Stable"]
    #[inline(always)]
    pub fn lfxtalnok(&mut self) -> LFXTALNOK_W {
        LFXTALNOK_W { w: self }
    }
    #[doc = "Bit 13 - HF Crystal Stable"]
    #[inline(always)]
    pub fn hfxtalok(&mut self) -> HFXTALOK_W {
        HFXTALOK_W { w: self }
    }
    #[doc = "Bit 14 - HF Crystal Not Stable"]
    #[inline(always)]
    pub fn hfxtalnok(&mut self) -> HFXTALNOK_W {
        HFXTALNOK_W { w: self }
    }
}
