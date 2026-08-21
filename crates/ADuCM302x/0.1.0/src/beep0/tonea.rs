#[doc = "Reader of register TONEA"]
pub type R = crate::R<u16, super::TONEA>;
#[doc = "Writer for register TONEA"]
pub type W = crate::W<u16, super::TONEA>;
#[doc = "Register TONEA `reset()`'s with value 0x01"]
impl crate::ResetValue for super::TONEA {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x01
    }
}
#[doc = "Reader of field `DUR`"]
pub type DUR_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DUR`"]
pub struct DUR_W<'a> {
    w: &'a mut W,
}
impl<'a> DUR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `FREQ`"]
pub type FREQ_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FREQ`"]
pub struct FREQ_W<'a> {
    w: &'a mut W,
}
impl<'a> FREQ_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x7f << 8)) | (((value as u16) & 0x7f) << 8);
        self.w
    }
}
#[doc = "Reader of field `DIS`"]
pub type DIS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DIS`"]
pub struct DIS_W<'a> {
    w: &'a mut W,
}
impl<'a> DIS_W<'a> {
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
    #[doc = "Bits 0:7 - Tone Duration"]
    #[inline(always)]
    pub fn dur(&self) -> DUR_R {
        DUR_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:14 - Tone Frequency"]
    #[inline(always)]
    pub fn freq(&self) -> FREQ_R {
        FREQ_R::new(((self.bits >> 8) & 0x7f) as u8)
    }
    #[doc = "Bit 15 - Output Disable"]
    #[inline(always)]
    pub fn dis(&self) -> DIS_R {
        DIS_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Tone Duration"]
    #[inline(always)]
    pub fn dur(&mut self) -> DUR_W {
        DUR_W { w: self }
    }
    #[doc = "Bits 8:14 - Tone Frequency"]
    #[inline(always)]
    pub fn freq(&mut self) -> FREQ_W {
        FREQ_W { w: self }
    }
    #[doc = "Bit 15 - Output Disable"]
    #[inline(always)]
    pub fn dis(&mut self) -> DIS_W {
        DIS_W { w: self }
    }
}
