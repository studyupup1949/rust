#[doc = "Reader of register RSC"]
pub type R = crate::R<u16, super::RSC>;
#[doc = "Writer for register RSC"]
pub type W = crate::W<u16, super::RSC>;
#[doc = "Register RSC `reset()`'s with value 0"]
impl crate::ResetValue for super::RSC {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `OENP`"]
pub type OENP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OENP`"]
pub struct OENP_W<'a> {
    w: &'a mut W,
}
impl<'a> OENP_W<'a> {
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
#[doc = "Reader of field `OENSP`"]
pub type OENSP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OENSP`"]
pub struct OENSP_W<'a> {
    w: &'a mut W,
}
impl<'a> OENSP_W<'a> {
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
#[doc = "Reader of field `DISRX`"]
pub type DISRX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DISRX`"]
pub struct DISRX_W<'a> {
    w: &'a mut W,
}
impl<'a> DISRX_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u16) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `DISTX`"]
pub type DISTX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DISTX`"]
pub struct DISTX_W<'a> {
    w: &'a mut W,
}
impl<'a> DISTX_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u16) & 0x01) << 3);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - SOUT_EN Polarity"]
    #[inline(always)]
    pub fn oenp(&self) -> OENP_R {
        OENP_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - SOUT_EN De-assert Before Full Stop Bit(s)"]
    #[inline(always)]
    pub fn oensp(&self) -> OENSP_R {
        OENSP_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Disable Rx When Transmitting"]
    #[inline(always)]
    pub fn disrx(&self) -> DISRX_R {
        DISRX_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Hold off Tx When Receiving"]
    #[inline(always)]
    pub fn distx(&self) -> DISTX_R {
        DISTX_R::new(((self.bits >> 3) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SOUT_EN Polarity"]
    #[inline(always)]
    pub fn oenp(&mut self) -> OENP_W {
        OENP_W { w: self }
    }
    #[doc = "Bit 1 - SOUT_EN De-assert Before Full Stop Bit(s)"]
    #[inline(always)]
    pub fn oensp(&mut self) -> OENSP_W {
        OENSP_W { w: self }
    }
    #[doc = "Bit 2 - Disable Rx When Transmitting"]
    #[inline(always)]
    pub fn disrx(&mut self) -> DISRX_W {
        DISRX_W { w: self }
    }
    #[doc = "Bit 3 - Hold off Tx When Receiving"]
    #[inline(always)]
    pub fn distx(&mut self) -> DISTX_W {
        DISTX_W { w: self }
    }
}
