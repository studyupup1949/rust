#[doc = "Reader of register CTL"]
pub type R = crate::R<u16, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u16, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0xe9"]
impl crate::ResetValue for super::CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0xe9
    }
}
#[doc = "Timer Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IRQ_A {
    #[doc = "0: WDT asserts reset when timed out"]
    RST = 0,
    #[doc = "1: WDT generates interrupt when timed out"]
    INT = 1,
}
impl From<IRQ_A> for bool {
    #[inline(always)]
    fn from(variant: IRQ_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `IRQ`"]
pub type IRQ_R = crate::R<bool, IRQ_A>;
impl IRQ_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> IRQ_A {
        match self.bits {
            false => IRQ_A::RST,
            true => IRQ_A::INT,
        }
    }
    #[doc = "Checks if the value of the field is `RST`"]
    #[inline(always)]
    pub fn is_rst(&self) -> bool {
        *self == IRQ_A::RST
    }
    #[doc = "Checks if the value of the field is `INT`"]
    #[inline(always)]
    pub fn is_int(&self) -> bool {
        *self == IRQ_A::INT
    }
}
#[doc = "Write proxy for field `IRQ`"]
pub struct IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: IRQ_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "WDT asserts reset when timed out"]
    #[inline(always)]
    pub fn rst(self) -> &'a mut W {
        self.variant(IRQ_A::RST)
    }
    #[doc = "WDT generates interrupt when timed out"]
    #[inline(always)]
    pub fn int(self) -> &'a mut W {
        self.variant(IRQ_A::INT)
    }
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
#[doc = "Prescaler\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PRE_A {
    #[doc = "0: Source clock/1"]
    DIV1 = 0,
    #[doc = "1: Source clock/16"]
    DIV16 = 1,
    #[doc = "2: Source clock/256 (default)"]
    DIV256 = 2,
}
impl From<PRE_A> for u8 {
    #[inline(always)]
    fn from(variant: PRE_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `PRE`"]
pub type PRE_R = crate::R<u8, PRE_A>;
impl PRE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, PRE_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(PRE_A::DIV1),
            1 => Val(PRE_A::DIV16),
            2 => Val(PRE_A::DIV256),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `DIV1`"]
    #[inline(always)]
    pub fn is_div1(&self) -> bool {
        *self == PRE_A::DIV1
    }
    #[doc = "Checks if the value of the field is `DIV16`"]
    #[inline(always)]
    pub fn is_div16(&self) -> bool {
        *self == PRE_A::DIV16
    }
    #[doc = "Checks if the value of the field is `DIV256`"]
    #[inline(always)]
    pub fn is_div256(&self) -> bool {
        *self == PRE_A::DIV256
    }
}
#[doc = "Write proxy for field `PRE`"]
pub struct PRE_W<'a> {
    w: &'a mut W,
}
impl<'a> PRE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PRE_A) -> &'a mut W {
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "Source clock/1"]
    #[inline(always)]
    pub fn div1(self) -> &'a mut W {
        self.variant(PRE_A::DIV1)
    }
    #[doc = "Source clock/16"]
    #[inline(always)]
    pub fn div16(self) -> &'a mut W {
        self.variant(PRE_A::DIV16)
    }
    #[doc = "Source clock/256 (default)"]
    #[inline(always)]
    pub fn div256(self) -> &'a mut W {
        self.variant(PRE_A::DIV256)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 2)) | (((value as u16) & 0x03) << 2);
        self.w
    }
}
#[doc = "Timer Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EN_A {
    #[doc = "0: WDT not enabled"]
    WDT_DIS = 0,
    #[doc = "1: WDT enabled"]
    WDT_EN = 1,
}
impl From<EN_A> for bool {
    #[inline(always)]
    fn from(variant: EN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `EN`"]
pub type EN_R = crate::R<bool, EN_A>;
impl EN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> EN_A {
        match self.bits {
            false => EN_A::WDT_DIS,
            true => EN_A::WDT_EN,
        }
    }
    #[doc = "Checks if the value of the field is `WDT_DIS`"]
    #[inline(always)]
    pub fn is_wdt_dis(&self) -> bool {
        *self == EN_A::WDT_DIS
    }
    #[doc = "Checks if the value of the field is `WDT_EN`"]
    #[inline(always)]
    pub fn is_wdt_en(&self) -> bool {
        *self == EN_A::WDT_EN
    }
}
#[doc = "Write proxy for field `EN`"]
pub struct EN_W<'a> {
    w: &'a mut W,
}
impl<'a> EN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: EN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "WDT not enabled"]
    #[inline(always)]
    pub fn wdt_dis(self) -> &'a mut W {
        self.variant(EN_A::WDT_DIS)
    }
    #[doc = "WDT enabled"]
    #[inline(always)]
    pub fn wdt_en(self) -> &'a mut W {
        self.variant(EN_A::WDT_EN)
    }
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
#[doc = "Timer Mode\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MODE_A {
    #[doc = "0: Free running mode"]
    FREE_RUN = 0,
    #[doc = "1: Periodic mode"]
    PERIODIC = 1,
}
impl From<MODE_A> for bool {
    #[inline(always)]
    fn from(variant: MODE_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `MODE`"]
pub type MODE_R = crate::R<bool, MODE_A>;
impl MODE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> MODE_A {
        match self.bits {
            false => MODE_A::FREE_RUN,
            true => MODE_A::PERIODIC,
        }
    }
    #[doc = "Checks if the value of the field is `FREE_RUN`"]
    #[inline(always)]
    pub fn is_free_run(&self) -> bool {
        *self == MODE_A::FREE_RUN
    }
    #[doc = "Checks if the value of the field is `PERIODIC`"]
    #[inline(always)]
    pub fn is_periodic(&self) -> bool {
        *self == MODE_A::PERIODIC
    }
}
#[doc = "Write proxy for field `MODE`"]
pub struct MODE_W<'a> {
    w: &'a mut W,
}
impl<'a> MODE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: MODE_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Free running mode"]
    #[inline(always)]
    pub fn free_run(self) -> &'a mut W {
        self.variant(MODE_A::FREE_RUN)
    }
    #[doc = "Periodic mode"]
    #[inline(always)]
    pub fn periodic(self) -> &'a mut W {
        self.variant(MODE_A::PERIODIC)
    }
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
#[doc = "Reader of field `SPARE`"]
pub type SPARE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPARE`"]
pub struct SPARE_W<'a> {
    w: &'a mut W,
}
impl<'a> SPARE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u16) & 0x01) << 7);
        self.w
    }
}
impl R {
    #[doc = "Bit 1 - Timer Interrupt"]
    #[inline(always)]
    pub fn irq(&self) -> IRQ_R {
        IRQ_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bits 2:3 - Prescaler"]
    #[inline(always)]
    pub fn pre(&self) -> PRE_R {
        PRE_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bit 5 - Timer Enable"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Timer Mode"]
    #[inline(always)]
    pub fn mode(&self) -> MODE_R {
        MODE_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Unused Spare Bit"]
    #[inline(always)]
    pub fn spare(&self) -> SPARE_R {
        SPARE_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Timer Interrupt"]
    #[inline(always)]
    pub fn irq(&mut self) -> IRQ_W {
        IRQ_W { w: self }
    }
    #[doc = "Bits 2:3 - Prescaler"]
    #[inline(always)]
    pub fn pre(&mut self) -> PRE_W {
        PRE_W { w: self }
    }
    #[doc = "Bit 5 - Timer Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 6 - Timer Mode"]
    #[inline(always)]
    pub fn mode(&mut self) -> MODE_W {
        MODE_W { w: self }
    }
    #[doc = "Bit 7 - Unused Spare Bit"]
    #[inline(always)]
    pub fn spare(&mut self) -> SPARE_W {
        SPARE_W { w: self }
    }
}
