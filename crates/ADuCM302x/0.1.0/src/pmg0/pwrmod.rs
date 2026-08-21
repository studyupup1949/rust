#[doc = "Reader of register PWRMOD"]
pub type R = crate::R<u32, super::PWRMOD>;
#[doc = "Writer for register PWRMOD"]
pub type W = crate::W<u32, super::PWRMOD>;
#[doc = "Register PWRMOD `reset()`'s with value 0"]
impl crate::ResetValue for super::PWRMOD {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Power Mode Bits\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum MODE_A {
    #[doc = "0: Flexi Mode"]
    FLEXI = 0,
    #[doc = "2: Hibernate Mode"]
    HIBERNATE = 2,
    #[doc = "3: Shutdown Mode"]
    SHUTDOWN = 3,
}
impl From<MODE_A> for u8 {
    #[inline(always)]
    fn from(variant: MODE_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `MODE`"]
pub type MODE_R = crate::R<u8, MODE_A>;
impl MODE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, MODE_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(MODE_A::FLEXI),
            2 => Val(MODE_A::HIBERNATE),
            3 => Val(MODE_A::SHUTDOWN),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `FLEXI`"]
    #[inline(always)]
    pub fn is_flexi(&self) -> bool {
        *self == MODE_A::FLEXI
    }
    #[doc = "Checks if the value of the field is `HIBERNATE`"]
    #[inline(always)]
    pub fn is_hibernate(&self) -> bool {
        *self == MODE_A::HIBERNATE
    }
    #[doc = "Checks if the value of the field is `SHUTDOWN`"]
    #[inline(always)]
    pub fn is_shutdown(&self) -> bool {
        *self == MODE_A::SHUTDOWN
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
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "Flexi Mode"]
    #[inline(always)]
    pub fn flexi(self) -> &'a mut W {
        self.variant(MODE_A::FLEXI)
    }
    #[doc = "Hibernate Mode"]
    #[inline(always)]
    pub fn hibernate(self) -> &'a mut W {
        self.variant(MODE_A::HIBERNATE)
    }
    #[doc = "Shutdown Mode"]
    #[inline(always)]
    pub fn shutdown(self) -> &'a mut W {
        self.variant(MODE_A::SHUTDOWN)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Monitor VBAT During Hibernate Mode. Monitors VBAT by Default\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MONVBATN_A {
    #[doc = "0: VBAT monitor enabled in PMG block."]
    VBAT_MONEN = 0,
    #[doc = "1: VBAT monitor disabled in PMG block."]
    VBAT_MONDIS = 1,
}
impl From<MONVBATN_A> for bool {
    #[inline(always)]
    fn from(variant: MONVBATN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `MONVBATN`"]
pub type MONVBATN_R = crate::R<bool, MONVBATN_A>;
impl MONVBATN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> MONVBATN_A {
        match self.bits {
            false => MONVBATN_A::VBAT_MONEN,
            true => MONVBATN_A::VBAT_MONDIS,
        }
    }
    #[doc = "Checks if the value of the field is `VBAT_MONEN`"]
    #[inline(always)]
    pub fn is_vbat_monen(&self) -> bool {
        *self == MONVBATN_A::VBAT_MONEN
    }
    #[doc = "Checks if the value of the field is `VBAT_MONDIS`"]
    #[inline(always)]
    pub fn is_vbat_mondis(&self) -> bool {
        *self == MONVBATN_A::VBAT_MONDIS
    }
}
#[doc = "Write proxy for field `MONVBATN`"]
pub struct MONVBATN_W<'a> {
    w: &'a mut W,
}
impl<'a> MONVBATN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: MONVBATN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "VBAT monitor enabled in PMG block."]
    #[inline(always)]
    pub fn vbat_monen(self) -> &'a mut W {
        self.variant(MONVBATN_A::VBAT_MONEN)
    }
    #[doc = "VBAT monitor disabled in PMG block."]
    #[inline(always)]
    pub fn vbat_mondis(self) -> &'a mut W {
        self.variant(MONVBATN_A::VBAT_MONDIS)
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Power Mode Bits"]
    #[inline(always)]
    pub fn mode(&self) -> MODE_R {
        MODE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bit 3 - Monitor VBAT During Hibernate Mode. Monitors VBAT by Default"]
    #[inline(always)]
    pub fn monvbatn(&self) -> MONVBATN_R {
        MONVBATN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Power Mode Bits"]
    #[inline(always)]
    pub fn mode(&mut self) -> MODE_W {
        MODE_W { w: self }
    }
    #[doc = "Bit 3 - Monitor VBAT During Hibernate Mode. Monitors VBAT by Default"]
    #[inline(always)]
    pub fn monvbatn(&mut self) -> MONVBATN_W {
        MONVBATN_W { w: self }
    }
}
