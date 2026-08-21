#[doc = "Reader of register CFG"]
pub type R = crate::R<u16, super::CFG>;
#[doc = "Writer for register CFG"]
pub type W = crate::W<u16, super::CFG>;
#[doc = "Register CFG `reset()`'s with value 0"]
impl crate::ResetValue for super::CFG {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `PWRUP`"]
pub type PWRUP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PWRUP`"]
pub struct PWRUP_W<'a> {
    w: &'a mut W,
}
impl<'a> PWRUP_W<'a> {
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
#[doc = "Select Vref as 1.25V or 2.5V\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VREFSEL_A {
    #[doc = "0: Vref = 2.5V"]
    V_2P5 = 0,
    #[doc = "1: Vref = 1.25V"]
    V_1P25 = 1,
}
impl From<VREFSEL_A> for bool {
    #[inline(always)]
    fn from(variant: VREFSEL_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `VREFSEL`"]
pub type VREFSEL_R = crate::R<bool, VREFSEL_A>;
impl VREFSEL_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> VREFSEL_A {
        match self.bits {
            false => VREFSEL_A::V_2P5,
            true => VREFSEL_A::V_1P25,
        }
    }
    #[doc = "Checks if the value of the field is `V_2P5`"]
    #[inline(always)]
    pub fn is_v_2p5(&self) -> bool {
        *self == VREFSEL_A::V_2P5
    }
    #[doc = "Checks if the value of the field is `V_1P25`"]
    #[inline(always)]
    pub fn is_v_1p25(&self) -> bool {
        *self == VREFSEL_A::V_1P25
    }
}
#[doc = "Write proxy for field `VREFSEL`"]
pub struct VREFSEL_W<'a> {
    w: &'a mut W,
}
impl<'a> VREFSEL_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: VREFSEL_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Vref = 2.5V"]
    #[inline(always)]
    pub fn v_2p5(self) -> &'a mut W {
        self.variant(VREFSEL_A::V_2P5)
    }
    #[doc = "Vref = 1.25V"]
    #[inline(always)]
    pub fn v_1p25(self) -> &'a mut W {
        self.variant(VREFSEL_A::V_1P25)
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
#[doc = "Enable Internal Reference Buffer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum REFBUFEN_A {
    #[doc = "0: External reference is used"]
    EXT_REF = 0,
    #[doc = "1: Reference buffer is enabled"]
    BUF_REF = 1,
}
impl From<REFBUFEN_A> for bool {
    #[inline(always)]
    fn from(variant: REFBUFEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `REFBUFEN`"]
pub type REFBUFEN_R = crate::R<bool, REFBUFEN_A>;
impl REFBUFEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> REFBUFEN_A {
        match self.bits {
            false => REFBUFEN_A::EXT_REF,
            true => REFBUFEN_A::BUF_REF,
        }
    }
    #[doc = "Checks if the value of the field is `EXT_REF`"]
    #[inline(always)]
    pub fn is_ext_ref(&self) -> bool {
        *self == REFBUFEN_A::EXT_REF
    }
    #[doc = "Checks if the value of the field is `BUF_REF`"]
    #[inline(always)]
    pub fn is_buf_ref(&self) -> bool {
        *self == REFBUFEN_A::BUF_REF
    }
}
#[doc = "Write proxy for field `REFBUFEN`"]
pub struct REFBUFEN_W<'a> {
    w: &'a mut W,
}
impl<'a> REFBUFEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: REFBUFEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "External reference is used"]
    #[inline(always)]
    pub fn ext_ref(self) -> &'a mut W {
        self.variant(REFBUFEN_A::EXT_REF)
    }
    #[doc = "Reference buffer is enabled"]
    #[inline(always)]
    pub fn buf_ref(self) -> &'a mut W {
        self.variant(REFBUFEN_A::BUF_REF)
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u16) & 0x01) << 2);
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `STARTCAL`"]
pub type STARTCAL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STARTCAL`"]
pub struct STARTCAL_W<'a> {
    w: &'a mut W,
}
impl<'a> STARTCAL_W<'a> {
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
#[doc = "Reader of field `RST`"]
pub type RST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RST`"]
pub struct RST_W<'a> {
    w: &'a mut W,
}
impl<'a> RST_W<'a> {
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
#[doc = "Reader of field `SINKEN`"]
pub type SINKEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SINKEN`"]
pub struct SINKEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SINKEN_W<'a> {
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
#[doc = "Reader of field `TMPEN`"]
pub type TMPEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TMPEN`"]
pub struct TMPEN_W<'a> {
    w: &'a mut W,
}
impl<'a> TMPEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `FAST_DISCH`"]
pub type FAST_DISCH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FAST_DISCH`"]
pub struct FAST_DISCH_W<'a> {
    w: &'a mut W,
}
impl<'a> FAST_DISCH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u16) & 0x01) << 9);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Powering up the ADC"]
    #[inline(always)]
    pub fn pwrup(&self) -> PWRUP_R {
        PWRUP_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Select Vref as 1.25V or 2.5V"]
    #[inline(always)]
    pub fn vrefsel(&self) -> VREFSEL_R {
        VREFSEL_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable Internal Reference Buffer"]
    #[inline(always)]
    pub fn refbufen(&self) -> REFBUFEN_R {
        REFBUFEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Enable ADC Subsystem"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Start a New Offset Calibration Cycle"]
    #[inline(always)]
    pub fn startcal(&self) -> STARTCAL_R {
        STARTCAL_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Reset"]
    #[inline(always)]
    pub fn rst(&self) -> RST_R {
        RST_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Enable Additional Sink Current Capability"]
    #[inline(always)]
    pub fn sinken(&self) -> SINKEN_R {
        SINKEN_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Power up Temperature Sensor"]
    #[inline(always)]
    pub fn tmpen(&self) -> TMPEN_R {
        TMPEN_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Fast Switchover of Vref from 2.5 to 1.25"]
    #[inline(always)]
    pub fn fast_disch(&self) -> FAST_DISCH_R {
        FAST_DISCH_R::new(((self.bits >> 9) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Powering up the ADC"]
    #[inline(always)]
    pub fn pwrup(&mut self) -> PWRUP_W {
        PWRUP_W { w: self }
    }
    #[doc = "Bit 1 - Select Vref as 1.25V or 2.5V"]
    #[inline(always)]
    pub fn vrefsel(&mut self) -> VREFSEL_W {
        VREFSEL_W { w: self }
    }
    #[doc = "Bit 2 - Enable Internal Reference Buffer"]
    #[inline(always)]
    pub fn refbufen(&mut self) -> REFBUFEN_W {
        REFBUFEN_W { w: self }
    }
    #[doc = "Bit 4 - Enable ADC Subsystem"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 5 - Start a New Offset Calibration Cycle"]
    #[inline(always)]
    pub fn startcal(&mut self) -> STARTCAL_W {
        STARTCAL_W { w: self }
    }
    #[doc = "Bit 6 - Reset"]
    #[inline(always)]
    pub fn rst(&mut self) -> RST_W {
        RST_W { w: self }
    }
    #[doc = "Bit 7 - Enable Additional Sink Current Capability"]
    #[inline(always)]
    pub fn sinken(&mut self) -> SINKEN_W {
        SINKEN_W { w: self }
    }
    #[doc = "Bit 8 - Power up Temperature Sensor"]
    #[inline(always)]
    pub fn tmpen(&mut self) -> TMPEN_W {
        TMPEN_W { w: self }
    }
    #[doc = "Bit 9 - Fast Switchover of Vref from 2.5 to 1.25"]
    #[inline(always)]
    pub fn fast_disch(&mut self) -> FAST_DISCH_W {
        FAST_DISCH_W { w: self }
    }
}
