#[doc = "Reader of register CR1"]
pub type R = crate::R<u16, super::CR1>;
#[doc = "Writer for register CR1"]
pub type W = crate::W<u16, super::CR1>;
#[doc = "Register CR1 `reset()`'s with value 0x01e0"]
impl crate::ResetValue for super::CR1 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x01e0
    }
}
#[doc = "Reader of field `CNTINTEN`"]
pub type CNTINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTINTEN`"]
pub struct CNTINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTINTEN_W<'a> {
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
#[doc = "Reader of field `PSINTEN`"]
pub type PSINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PSINTEN`"]
pub struct PSINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> PSINTEN_W<'a> {
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
#[doc = "Reader of field `TRMINTEN`"]
pub type TRMINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TRMINTEN`"]
pub struct TRMINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> TRMINTEN_W<'a> {
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
#[doc = "Reader of field `CNTROLLINTEN`"]
pub type CNTROLLINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTROLLINTEN`"]
pub struct CNTROLLINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTROLLINTEN_W<'a> {
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
#[doc = "Reader of field `CNTMOD60ROLLINTEN`"]
pub type CNTMOD60ROLLINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTMOD60ROLLINTEN`"]
pub struct CNTMOD60ROLLINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTMOD60ROLLINTEN_W<'a> {
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
#[doc = "Reader of field `PRESCALE2EXP`"]
pub type PRESCALE2EXP_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PRESCALE2EXP`"]
pub struct PRESCALE2EXP_W<'a> {
    w: &'a mut W,
}
impl<'a> PRESCALE2EXP_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 5)) | (((value as u16) & 0x0f) << 5);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable for the RTC Count Interrupt Source"]
    #[inline(always)]
    pub fn cntinten(&self) -> CNTINTEN_R {
        CNTINTEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable for the Prescaled, Modulo-1 Interrupt Source, in SR2:RTCPSINT"]
    #[inline(always)]
    pub fn psinten(&self) -> PSINTEN_R {
        PSINTEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable for the RTC Trim Interrupt Source, in SR2:RTCTRMINT"]
    #[inline(always)]
    pub fn trminten(&self) -> TRMINTEN_R {
        TRMINTEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Enable for the RTC Count Roll-Over Interrupt Source, in SR2:RTCCNTROLLINT"]
    #[inline(always)]
    pub fn cntrollinten(&self) -> CNTROLLINTEN_R {
        CNTROLLINTEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Enable for the RTC Modulo-60 Count Roll-Over Interrupt Source, in SR2:RTCCNTMOD60ROLLINT"]
    #[inline(always)]
    pub fn cntmod60rollinten(&self) -> CNTMOD60ROLLINTEN_R {
        CNTMOD60ROLLINTEN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 5:8 - Prescale Power of 2 Division Factor for the RTC Base Clock"]
    #[inline(always)]
    pub fn prescale2exp(&self) -> PRESCALE2EXP_R {
        PRESCALE2EXP_R::new(((self.bits >> 5) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Enable for the RTC Count Interrupt Source"]
    #[inline(always)]
    pub fn cntinten(&mut self) -> CNTINTEN_W {
        CNTINTEN_W { w: self }
    }
    #[doc = "Bit 1 - Enable for the Prescaled, Modulo-1 Interrupt Source, in SR2:RTCPSINT"]
    #[inline(always)]
    pub fn psinten(&mut self) -> PSINTEN_W {
        PSINTEN_W { w: self }
    }
    #[doc = "Bit 2 - Enable for the RTC Trim Interrupt Source, in SR2:RTCTRMINT"]
    #[inline(always)]
    pub fn trminten(&mut self) -> TRMINTEN_W {
        TRMINTEN_W { w: self }
    }
    #[doc = "Bit 3 - Enable for the RTC Count Roll-Over Interrupt Source, in SR2:RTCCNTROLLINT"]
    #[inline(always)]
    pub fn cntrollinten(&mut self) -> CNTROLLINTEN_W {
        CNTROLLINTEN_W { w: self }
    }
    #[doc = "Bit 4 - Enable for the RTC Modulo-60 Count Roll-Over Interrupt Source, in SR2:RTCCNTMOD60ROLLINT"]
    #[inline(always)]
    pub fn cntmod60rollinten(&mut self) -> CNTMOD60ROLLINTEN_W {
        CNTMOD60ROLLINTEN_W { w: self }
    }
    #[doc = "Bits 5:8 - Prescale Power of 2 Division Factor for the RTC Base Clock"]
    #[inline(always)]
    pub fn prescale2exp(&mut self) -> PRESCALE2EXP_W {
        PRESCALE2EXP_W { w: self }
    }
}
