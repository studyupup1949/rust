#[doc = "Reader of register SR0"]
pub type R = crate::R<u16, super::SR0>;
#[doc = "Writer for register SR0"]
pub type W = crate::W<u16, super::SR0>;
#[doc = "Register SR0 `reset()`'s with value 0x3f80"]
impl crate::ResetValue for super::SR0 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x3f80
    }
}
#[doc = "Reader of field `ALMINT`"]
pub type ALMINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALMINT`"]
pub struct ALMINT_W<'a> {
    w: &'a mut W,
}
impl<'a> ALMINT_W<'a> {
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
#[doc = "Reader of field `MOD60ALMINT`"]
pub type MOD60ALMINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MOD60ALMINT`"]
pub struct MOD60ALMINT_W<'a> {
    w: &'a mut W,
}
impl<'a> MOD60ALMINT_W<'a> {
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
#[doc = "Reader of field `ISOINT`"]
pub type ISOINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ISOINT`"]
pub struct ISOINT_W<'a> {
    w: &'a mut W,
}
impl<'a> ISOINT_W<'a> {
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
#[doc = "Reader of field `WPNDERRINT`"]
pub type WPNDERRINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WPNDERRINT`"]
pub struct WPNDERRINT_W<'a> {
    w: &'a mut W,
}
impl<'a> WPNDERRINT_W<'a> {
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
#[doc = "Reader of field `WSYNCINT`"]
pub type WSYNCINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WSYNCINT`"]
pub struct WSYNCINT_W<'a> {
    w: &'a mut W,
}
impl<'a> WSYNCINT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u16) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `WPNDINT`"]
pub type WPNDINT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WPNDINT`"]
pub struct WPNDINT_W<'a> {
    w: &'a mut W,
}
impl<'a> WPNDINT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u16) & 0x01) << 6);
        self.w
    }
}
#[doc = "Reader of field `WSYNCCR0`"]
pub type WSYNCCR0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCSR0`"]
pub type WSYNCSR0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCNT0`"]
pub type WSYNCCNT0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCNT1`"]
pub type WSYNCCNT1_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCALM0`"]
pub type WSYNCALM0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCALM1`"]
pub type WSYNCALM1_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCTRM`"]
pub type WSYNCTRM_R = crate::R<bool, bool>;
#[doc = "Reader of field `ISOENB`"]
pub type ISOENB_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 1 - Alarm Interrupt Source"]
    #[inline(always)]
    pub fn almint(&self) -> ALMINT_R {
        ALMINT_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Modulo-60 RTC Alarm Interrupt Source"]
    #[inline(always)]
    pub fn mod60almint(&self) -> MOD60ALMINT_R {
        MOD60ALMINT_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - RTC Power-Domain Isolation Interrupt Source"]
    #[inline(always)]
    pub fn isoint(&self) -> ISOINT_R {
        ISOINT_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Write Pending Error Interrupt Source"]
    #[inline(always)]
    pub fn wpnderrint(&self) -> WPNDERRINT_R {
        WPNDERRINT_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Write Synchronisation Interrupt"]
    #[inline(always)]
    pub fn wsyncint(&self) -> WSYNCINT_R {
        WSYNCINT_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Write Pending Interrupt"]
    #[inline(always)]
    pub fn wpndint(&self) -> WPNDINT_R {
        WPNDINT_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Synchronisation Status of Posted Writes to CR0"]
    #[inline(always)]
    pub fn wsynccr0(&self) -> WSYNCCR0_R {
        WSYNCCR0_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Synchronisation Status of Posted Writes to SR0"]
    #[inline(always)]
    pub fn wsyncsr0(&self) -> WSYNCSR0_R {
        WSYNCSR0_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Synchronisation Status of Posted Writes to CNT0"]
    #[inline(always)]
    pub fn wsynccnt0(&self) -> WSYNCCNT0_R {
        WSYNCCNT0_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Synchronisation Status of Posted Writes to CNT1"]
    #[inline(always)]
    pub fn wsynccnt1(&self) -> WSYNCCNT1_R {
        WSYNCCNT1_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Synchronisation Status of Posted Writes to ALM0"]
    #[inline(always)]
    pub fn wsyncalm0(&self) -> WSYNCALM0_R {
        WSYNCALM0_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Synchronisation Status of Posted Writes to ALM1"]
    #[inline(always)]
    pub fn wsyncalm1(&self) -> WSYNCALM1_R {
        WSYNCALM1_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Synchronisation Status of Posted Writes to TRM"]
    #[inline(always)]
    pub fn wsynctrm(&self) -> WSYNCTRM_R {
        WSYNCTRM_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Visibility of 32kHz Sourced Registers"]
    #[inline(always)]
    pub fn isoenb(&self) -> ISOENB_R {
        ISOENB_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Alarm Interrupt Source"]
    #[inline(always)]
    pub fn almint(&mut self) -> ALMINT_W {
        ALMINT_W { w: self }
    }
    #[doc = "Bit 2 - Modulo-60 RTC Alarm Interrupt Source"]
    #[inline(always)]
    pub fn mod60almint(&mut self) -> MOD60ALMINT_W {
        MOD60ALMINT_W { w: self }
    }
    #[doc = "Bit 3 - RTC Power-Domain Isolation Interrupt Source"]
    #[inline(always)]
    pub fn isoint(&mut self) -> ISOINT_W {
        ISOINT_W { w: self }
    }
    #[doc = "Bit 4 - Write Pending Error Interrupt Source"]
    #[inline(always)]
    pub fn wpnderrint(&mut self) -> WPNDERRINT_W {
        WPNDERRINT_W { w: self }
    }
    #[doc = "Bit 5 - Write Synchronisation Interrupt"]
    #[inline(always)]
    pub fn wsyncint(&mut self) -> WSYNCINT_W {
        WSYNCINT_W { w: self }
    }
    #[doc = "Bit 6 - Write Pending Interrupt"]
    #[inline(always)]
    pub fn wpndint(&mut self) -> WPNDINT_W {
        WPNDINT_W { w: self }
    }
}
