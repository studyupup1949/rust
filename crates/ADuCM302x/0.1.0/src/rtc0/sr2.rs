#[doc = "Reader of register SR2"]
pub type R = crate::R<u16, super::SR2>;
#[doc = "Writer for register SR2"]
pub type W = crate::W<u16, super::SR2>;
#[doc = "Register SR2 `reset()`'s with value 0xc000"]
impl crate::ResetValue for super::SR2 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0xc000
    }
}
#[doc = "Reader of field `CNTINT`"]
pub type CNTINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTINT`"]
pub struct CNTINT_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTINT_W<'a> {
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
#[doc = "Reader of field `PSINT`"]
pub type PSINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PSINT`"]
pub struct PSINT_W<'a> {
    w: &'a mut W,
}
impl<'a> PSINT_W<'a> {
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
#[doc = "Reader of field `TRMINT`"]
pub type TRMINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TRMINT`"]
pub struct TRMINT_W<'a> {
    w: &'a mut W,
}
impl<'a> TRMINT_W<'a> {
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
#[doc = "Reader of field `CNTROLLINT`"]
pub type CNTROLLINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTROLLINT`"]
pub struct CNTROLLINT_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTROLLINT_W<'a> {
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
#[doc = "Reader of field `CNTMOD60ROLLINT`"]
pub type CNTMOD60ROLLINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTMOD60ROLLINT`"]
pub struct CNTMOD60ROLLINT_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTMOD60ROLLINT_W<'a> {
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
#[doc = "Reader of field `CNTROLL`"]
pub type CNTROLL_R = crate::R<bool, bool>;
#[doc = "Reader of field `CNTMOD60ROLL`"]
pub type CNTMOD60ROLL_R = crate::R<bool, bool>;
#[doc = "Reader of field `TRMBDYMIR`"]
pub type TRMBDYMIR_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDCR1MIR`"]
pub type WPNDCR1MIR_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDALM2MIR`"]
pub type WPNDALM2MIR_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCR1MIR`"]
pub type WSYNCCR1MIR_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCALM2MIR`"]
pub type WSYNCALM2MIR_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - RTC Count Interrupt Source"]
    #[inline(always)]
    pub fn cntint(&self) -> CNTINT_R {
        CNTINT_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - RTC Prescaled, Modulo-1 Boundary Interrupt Source"]
    #[inline(always)]
    pub fn psint(&self) -> PSINT_R {
        PSINT_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - RTC Trim Interrupt Source"]
    #[inline(always)]
    pub fn trmint(&self) -> TRMINT_R {
        TRMINT_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - RTC Count Roll-Over Interrupt Source"]
    #[inline(always)]
    pub fn cntrollint(&self) -> CNTROLLINT_R {
        CNTROLLINT_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - RTC Modulo-60 Count Roll-Over Interrupt Source"]
    #[inline(always)]
    pub fn cntmod60rollint(&self) -> CNTMOD60ROLLINT_R {
        CNTMOD60ROLLINT_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - RTC Count Roll-Over"]
    #[inline(always)]
    pub fn cntroll(&self) -> CNTROLL_R {
        CNTROLL_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - RTC Count Modulo-60 Roll-Over"]
    #[inline(always)]
    pub fn cntmod60roll(&self) -> CNTMOD60ROLL_R {
        CNTMOD60ROLL_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Mirror of MOD:RTCTRMBDY"]
    #[inline(always)]
    pub fn trmbdymir(&self) -> TRMBDYMIR_R {
        TRMBDYMIR_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Pending Status of Posted Writes to CR1"]
    #[inline(always)]
    pub fn wpndcr1mir(&self) -> WPNDCR1MIR_R {
        WPNDCR1MIR_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Pending Status of Posted Writes to ALM2"]
    #[inline(always)]
    pub fn wpndalm2mir(&self) -> WPNDALM2MIR_R {
        WPNDALM2MIR_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Synchronization Status of Posted Writes to CR1"]
    #[inline(always)]
    pub fn wsynccr1mir(&self) -> WSYNCCR1MIR_R {
        WSYNCCR1MIR_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Synchronization Status of Posted Writes to ALM2"]
    #[inline(always)]
    pub fn wsyncalm2mir(&self) -> WSYNCALM2MIR_R {
        WSYNCALM2MIR_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RTC Count Interrupt Source"]
    #[inline(always)]
    pub fn cntint(&mut self) -> CNTINT_W {
        CNTINT_W { w: self }
    }
    #[doc = "Bit 1 - RTC Prescaled, Modulo-1 Boundary Interrupt Source"]
    #[inline(always)]
    pub fn psint(&mut self) -> PSINT_W {
        PSINT_W { w: self }
    }
    #[doc = "Bit 2 - RTC Trim Interrupt Source"]
    #[inline(always)]
    pub fn trmint(&mut self) -> TRMINT_W {
        TRMINT_W { w: self }
    }
    #[doc = "Bit 3 - RTC Count Roll-Over Interrupt Source"]
    #[inline(always)]
    pub fn cntrollint(&mut self) -> CNTROLLINT_W {
        CNTROLLINT_W { w: self }
    }
    #[doc = "Bit 4 - RTC Modulo-60 Count Roll-Over Interrupt Source"]
    #[inline(always)]
    pub fn cntmod60rollint(&mut self) -> CNTMOD60ROLLINT_W {
        CNTMOD60ROLLINT_W { w: self }
    }
}
