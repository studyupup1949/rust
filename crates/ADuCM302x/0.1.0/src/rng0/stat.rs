#[doc = "Reader of register STAT"]
pub type R = crate::R<u16, super::STAT>;
#[doc = "Writer for register STAT"]
pub type W = crate::W<u16, super::STAT>;
#[doc = "Register STAT `reset()`'s with value 0"]
impl crate::ResetValue for super::STAT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `RNRDY`"]
pub type RNRDY_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RNRDY`"]
pub struct RNRDY_W<'a> {
    w: &'a mut W,
}
impl<'a> RNRDY_W<'a> {
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
#[doc = "Reader of field `STUCK`"]
pub type STUCK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STUCK`"]
pub struct STUCK_W<'a> {
    w: &'a mut W,
}
impl<'a> STUCK_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Random Number Ready"]
    #[inline(always)]
    pub fn rnrdy(&self) -> RNRDY_R {
        RNRDY_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Sampled Data Stuck High or Low"]
    #[inline(always)]
    pub fn stuck(&self) -> STUCK_R {
        STUCK_R::new(((self.bits >> 1) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Random Number Ready"]
    #[inline(always)]
    pub fn rnrdy(&mut self) -> RNRDY_W {
        RNRDY_W { w: self }
    }
    #[doc = "Bit 1 - Sampled Data Stuck High or Low"]
    #[inline(always)]
    pub fn stuck(&mut self) -> STUCK_W {
        STUCK_W { w: self }
    }
}
