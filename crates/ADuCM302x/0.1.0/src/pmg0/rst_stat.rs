#[doc = "Reader of register RST_STAT"]
pub type R = crate::R<u32, super::RST_STAT>;
#[doc = "Writer for register RST_STAT"]
pub type W = crate::W<u32, super::RST_STAT>;
#[doc = "Register RST_STAT `reset()`'s with value 0"]
impl crate::ResetValue for super::RST_STAT {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `POR`"]
pub type POR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `POR`"]
pub struct POR_W<'a> {
    w: &'a mut W,
}
impl<'a> POR_W<'a> {
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
#[doc = "Reader of field `EXTRST`"]
pub type EXTRST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EXTRST`"]
pub struct EXTRST_W<'a> {
    w: &'a mut W,
}
impl<'a> EXTRST_W<'a> {
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
#[doc = "Reader of field `WDRST`"]
pub type WDRST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WDRST`"]
pub struct WDRST_W<'a> {
    w: &'a mut W,
}
impl<'a> WDRST_W<'a> {
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
#[doc = "Reader of field `SWRST`"]
pub type SWRST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SWRST`"]
pub struct SWRST_W<'a> {
    w: &'a mut W,
}
impl<'a> SWRST_W<'a> {
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
#[doc = "Power-on-Reset Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PORSRC_A {
    #[doc = "0: POR triggered because VBAT drops below Fail Safe"]
    FAILSAFE_HV = 0,
    #[doc = "1: POR trigger because VBAT supply (VBAT < 1.7 V)"]
    RST_VBAT = 1,
    #[doc = "2: POR triggered because VDD supply (VDD < 1.08 V)"]
    RST_VREG = 2,
    #[doc = "3: POR triggered because VREG drops below Fail Safe"]
    FAILSAFE_LV = 3,
}
impl From<PORSRC_A> for u8 {
    #[inline(always)]
    fn from(variant: PORSRC_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `PORSRC`"]
pub type PORSRC_R = crate::R<u8, PORSRC_A>;
impl PORSRC_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PORSRC_A {
        match self.bits {
            0 => PORSRC_A::FAILSAFE_HV,
            1 => PORSRC_A::RST_VBAT,
            2 => PORSRC_A::RST_VREG,
            3 => PORSRC_A::FAILSAFE_LV,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `FAILSAFE_HV`"]
    #[inline(always)]
    pub fn is_failsafe_hv(&self) -> bool {
        *self == PORSRC_A::FAILSAFE_HV
    }
    #[doc = "Checks if the value of the field is `RST_VBAT`"]
    #[inline(always)]
    pub fn is_rst_vbat(&self) -> bool {
        *self == PORSRC_A::RST_VBAT
    }
    #[doc = "Checks if the value of the field is `RST_VREG`"]
    #[inline(always)]
    pub fn is_rst_vreg(&self) -> bool {
        *self == PORSRC_A::RST_VREG
    }
    #[doc = "Checks if the value of the field is `FAILSAFE_LV`"]
    #[inline(always)]
    pub fn is_failsafe_lv(&self) -> bool {
        *self == PORSRC_A::FAILSAFE_LV
    }
}
impl R {
    #[doc = "Bit 0 - Power-on-Reset"]
    #[inline(always)]
    pub fn por(&self) -> POR_R {
        POR_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - External Reset"]
    #[inline(always)]
    pub fn extrst(&self) -> EXTRST_R {
        EXTRST_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Watchdog Time-out Reset"]
    #[inline(always)]
    pub fn wdrst(&self) -> WDRST_R {
        WDRST_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&self) -> SWRST_R {
        SWRST_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:5 - Power-on-Reset Source"]
    #[inline(always)]
    pub fn porsrc(&self) -> PORSRC_R {
        PORSRC_R::new(((self.bits >> 4) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Power-on-Reset"]
    #[inline(always)]
    pub fn por(&mut self) -> POR_W {
        POR_W { w: self }
    }
    #[doc = "Bit 1 - External Reset"]
    #[inline(always)]
    pub fn extrst(&mut self) -> EXTRST_W {
        EXTRST_W { w: self }
    }
    #[doc = "Bit 2 - Watchdog Time-out Reset"]
    #[inline(always)]
    pub fn wdrst(&mut self) -> WDRST_W {
        WDRST_W { w: self }
    }
    #[doc = "Bit 3 - Software Reset"]
    #[inline(always)]
    pub fn swrst(&mut self) -> SWRST_W {
        SWRST_W { w: self }
    }
}
