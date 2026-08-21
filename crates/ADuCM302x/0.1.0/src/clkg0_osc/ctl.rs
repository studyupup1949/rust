#[doc = "Reader of register CTL"]
pub type R = crate::R<u32, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u32, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0x02"]
impl crate::ResetValue for super::CTL {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x02
    }
}
#[doc = "Reader of field `LFCLKMUX`"]
pub type LFCLKMUX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFCLKMUX`"]
pub struct LFCLKMUX_W<'a> {
    w: &'a mut W,
}
impl<'a> LFCLKMUX_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `HFOSCEN`"]
pub type HFOSCEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HFOSCEN`"]
pub struct HFOSCEN_W<'a> {
    w: &'a mut W,
}
impl<'a> HFOSCEN_W<'a> {
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
#[doc = "Reader of field `LFXTALEN`"]
pub type LFXTALEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTALEN`"]
pub struct LFXTALEN_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTALEN_W<'a> {
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
#[doc = "Reader of field `HFXTALEN`"]
pub type HFXTALEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HFXTALEN`"]
pub struct HFXTALEN_W<'a> {
    w: &'a mut W,
}
impl<'a> HFXTALEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `LFXTAL_BYPASS`"]
pub type LFXTAL_BYPASS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTAL_BYPASS`"]
pub struct LFXTAL_BYPASS_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTAL_BYPASS_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `LFXTAL_MON_EN`"]
pub type LFXTAL_MON_EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTAL_MON_EN`"]
pub struct LFXTAL_MON_EN_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTAL_MON_EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `LFOSCOK`"]
pub type LFOSCOK_R = crate::R<bool, bool>;
#[doc = "Reader of field `HFOSCOK`"]
pub type HFOSCOK_R = crate::R<bool, bool>;
#[doc = "Reader of field `LFXTALOK`"]
pub type LFXTALOK_R = crate::R<bool, bool>;
#[doc = "Reader of field `HFXTALOK`"]
pub type HFXTALOK_R = crate::R<bool, bool>;
#[doc = "LFXTAL Not Stable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LFXTAL_MON_FAIL_STAT_A {
    #[doc = "0: LFXTAL is running fine"]
    LFXTAL_RUNNING = 0,
    #[doc = "1: LFXTAL is not running"]
    LFXTAL_NOTRUNNING = 1,
}
impl From<LFXTAL_MON_FAIL_STAT_A> for bool {
    #[inline(always)]
    fn from(variant: LFXTAL_MON_FAIL_STAT_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `LFXTAL_MON_FAIL_STAT`"]
pub type LFXTAL_MON_FAIL_STAT_R = crate::R<bool, LFXTAL_MON_FAIL_STAT_A>;
impl LFXTAL_MON_FAIL_STAT_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> LFXTAL_MON_FAIL_STAT_A {
        match self.bits {
            false => LFXTAL_MON_FAIL_STAT_A::LFXTAL_RUNNING,
            true => LFXTAL_MON_FAIL_STAT_A::LFXTAL_NOTRUNNING,
        }
    }
    #[doc = "Checks if the value of the field is `LFXTAL_RUNNING`"]
    #[inline(always)]
    pub fn is_lfxtal_running(&self) -> bool {
        *self == LFXTAL_MON_FAIL_STAT_A::LFXTAL_RUNNING
    }
    #[doc = "Checks if the value of the field is `LFXTAL_NOTRUNNING`"]
    #[inline(always)]
    pub fn is_lfxtal_notrunning(&self) -> bool {
        *self == LFXTAL_MON_FAIL_STAT_A::LFXTAL_NOTRUNNING
    }
}
#[doc = "Write proxy for field `LFXTAL_MON_FAIL_STAT`"]
pub struct LFXTAL_MON_FAIL_STAT_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTAL_MON_FAIL_STAT_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: LFXTAL_MON_FAIL_STAT_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "LFXTAL is running fine"]
    #[inline(always)]
    pub fn lfxtal_running(self) -> &'a mut W {
        self.variant(LFXTAL_MON_FAIL_STAT_A::LFXTAL_RUNNING)
    }
    #[doc = "LFXTAL is not running"]
    #[inline(always)]
    pub fn lfxtal_notrunning(self) -> &'a mut W {
        self.variant(LFXTAL_MON_FAIL_STAT_A::LFXTAL_NOTRUNNING)
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
        self.w.bits = (self.w.bits & !(0x01 << 31)) | (((value as u32) & 0x01) << 31);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - 32kHz Clock Select Mux"]
    #[inline(always)]
    pub fn lfclkmux(&self) -> LFCLKMUX_R {
        LFCLKMUX_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - High Frequency Internal Oscillator Enable"]
    #[inline(always)]
    pub fn hfoscen(&self) -> HFOSCEN_R {
        HFOSCEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Low Frequency Crystal Oscillator Enable"]
    #[inline(always)]
    pub fn lfxtalen(&self) -> LFXTALEN_R {
        LFXTALEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - High Frequency Crystal Oscillator Enable"]
    #[inline(always)]
    pub fn hfxtalen(&self) -> HFXTALEN_R {
        HFXTALEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Low Frequency Crystal Oscillator Bypass"]
    #[inline(always)]
    pub fn lfxtal_bypass(&self) -> LFXTAL_BYPASS_R {
        LFXTAL_BYPASS_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - LFXTAL Clock Monitor and Clock Fail Interrupt Enable"]
    #[inline(always)]
    pub fn lfxtal_mon_en(&self) -> LFXTAL_MON_EN_R {
        LFXTAL_MON_EN_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Status of LFOSC Oscillator"]
    #[inline(always)]
    pub fn lfoscok(&self) -> LFOSCOK_R {
        LFOSCOK_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Status of HFOSC"]
    #[inline(always)]
    pub fn hfoscok(&self) -> HFOSCOK_R {
        HFOSCOK_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Status of LFXTAL Oscillator"]
    #[inline(always)]
    pub fn lfxtalok(&self) -> LFXTALOK_R {
        LFXTALOK_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Status of HFXTAL Oscillator"]
    #[inline(always)]
    pub fn hfxtalok(&self) -> HFXTALOK_R {
        HFXTALOK_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 31 - LFXTAL Not Stable"]
    #[inline(always)]
    pub fn lfxtal_mon_fail_stat(&self) -> LFXTAL_MON_FAIL_STAT_R {
        LFXTAL_MON_FAIL_STAT_R::new(((self.bits >> 31) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - 32kHz Clock Select Mux"]
    #[inline(always)]
    pub fn lfclkmux(&mut self) -> LFCLKMUX_W {
        LFCLKMUX_W { w: self }
    }
    #[doc = "Bit 1 - High Frequency Internal Oscillator Enable"]
    #[inline(always)]
    pub fn hfoscen(&mut self) -> HFOSCEN_W {
        HFOSCEN_W { w: self }
    }
    #[doc = "Bit 2 - Low Frequency Crystal Oscillator Enable"]
    #[inline(always)]
    pub fn lfxtalen(&mut self) -> LFXTALEN_W {
        LFXTALEN_W { w: self }
    }
    #[doc = "Bit 3 - High Frequency Crystal Oscillator Enable"]
    #[inline(always)]
    pub fn hfxtalen(&mut self) -> HFXTALEN_W {
        HFXTALEN_W { w: self }
    }
    #[doc = "Bit 4 - Low Frequency Crystal Oscillator Bypass"]
    #[inline(always)]
    pub fn lfxtal_bypass(&mut self) -> LFXTAL_BYPASS_W {
        LFXTAL_BYPASS_W { w: self }
    }
    #[doc = "Bit 5 - LFXTAL Clock Monitor and Clock Fail Interrupt Enable"]
    #[inline(always)]
    pub fn lfxtal_mon_en(&mut self) -> LFXTAL_MON_EN_W {
        LFXTAL_MON_EN_W { w: self }
    }
    #[doc = "Bit 31 - LFXTAL Not Stable"]
    #[inline(always)]
    pub fn lfxtal_mon_fail_stat(&mut self) -> LFXTAL_MON_FAIL_STAT_W {
        LFXTAL_MON_FAIL_STAT_W { w: self }
    }
}
