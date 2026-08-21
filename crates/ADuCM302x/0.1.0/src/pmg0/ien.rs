#[doc = "Reader of register IEN"]
pub type R = crate::R<u32, super::IEN>;
#[doc = "Writer for register IEN"]
pub type W = crate::W<u32, super::IEN>;
#[doc = "Register IEN `reset()`'s with value 0"]
impl crate::ResetValue for super::IEN {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `VBAT`"]
pub type VBAT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `VBAT`"]
pub struct VBAT_W<'a> {
    w: &'a mut W,
}
impl<'a> VBAT_W<'a> {
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
#[doc = "Reader of field `VREGUNDR`"]
pub type VREGUNDR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `VREGUNDR`"]
pub struct VREGUNDR_W<'a> {
    w: &'a mut W,
}
impl<'a> VREGUNDR_W<'a> {
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
#[doc = "Reader of field `VREGOVR`"]
pub type VREGOVR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `VREGOVR`"]
pub struct VREGOVR_W<'a> {
    w: &'a mut W,
}
impl<'a> VREGOVR_W<'a> {
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
#[doc = "Battery Monitor Range\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum RANGEBAT_A {
    #[doc = "0: Configure to generate interrupt if VBAT > 2.75 V"]
    REGION1 = 0,
    #[doc = "1: Configure to generate interrupt if VBAT between 2.75 V - 1.6 V"]
    REGION2 = 1,
    #[doc = "2: Configure to generate interrupt if VBAT between 2.3 V - 1.6 V"]
    REGION3 = 2,
    #[doc = "3: N/A"]
    NA = 3,
}
impl From<RANGEBAT_A> for u8 {
    #[inline(always)]
    fn from(variant: RANGEBAT_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `RANGEBAT`"]
pub type RANGEBAT_R = crate::R<u8, RANGEBAT_A>;
impl RANGEBAT_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> RANGEBAT_A {
        match self.bits {
            0 => RANGEBAT_A::REGION1,
            1 => RANGEBAT_A::REGION2,
            2 => RANGEBAT_A::REGION3,
            3 => RANGEBAT_A::NA,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `REGION1`"]
    #[inline(always)]
    pub fn is_region1(&self) -> bool {
        *self == RANGEBAT_A::REGION1
    }
    #[doc = "Checks if the value of the field is `REGION2`"]
    #[inline(always)]
    pub fn is_region2(&self) -> bool {
        *self == RANGEBAT_A::REGION2
    }
    #[doc = "Checks if the value of the field is `REGION3`"]
    #[inline(always)]
    pub fn is_region3(&self) -> bool {
        *self == RANGEBAT_A::REGION3
    }
    #[doc = "Checks if the value of the field is `NA`"]
    #[inline(always)]
    pub fn is_na(&self) -> bool {
        *self == RANGEBAT_A::NA
    }
}
#[doc = "Write proxy for field `RANGEBAT`"]
pub struct RANGEBAT_W<'a> {
    w: &'a mut W,
}
impl<'a> RANGEBAT_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: RANGEBAT_A) -> &'a mut W {
        {
            self.bits(variant.into())
        }
    }
    #[doc = "Configure to generate interrupt if VBAT > 2.75 V"]
    #[inline(always)]
    pub fn region1(self) -> &'a mut W {
        self.variant(RANGEBAT_A::REGION1)
    }
    #[doc = "Configure to generate interrupt if VBAT between 2.75 V - 1.6 V"]
    #[inline(always)]
    pub fn region2(self) -> &'a mut W {
        self.variant(RANGEBAT_A::REGION2)
    }
    #[doc = "Configure to generate interrupt if VBAT between 2.3 V - 1.6 V"]
    #[inline(always)]
    pub fn region3(self) -> &'a mut W {
        self.variant(RANGEBAT_A::REGION3)
    }
    #[doc = "N/A"]
    #[inline(always)]
    pub fn na(self) -> &'a mut W {
        self.variant(RANGEBAT_A::NA)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 8)) | (((value as u32) & 0x03) << 8);
        self.w
    }
}
#[doc = "Reader of field `IENBAT`"]
pub type IENBAT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENBAT`"]
pub struct IENBAT_W<'a> {
    w: &'a mut W,
}
impl<'a> IENBAT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable Interrupt for VBAT"]
    #[inline(always)]
    pub fn vbat(&self) -> VBAT_R {
        VBAT_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable Interrupt When VREG Undervoltage: Below 1V"]
    #[inline(always)]
    pub fn vregundr(&self) -> VREGUNDR_R {
        VREGUNDR_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable Interrupt When VREG Overvoltage: Above 1.32V"]
    #[inline(always)]
    pub fn vregovr(&self) -> VREGOVR_R {
        VREGOVR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bits 8:9 - Battery Monitor Range"]
    #[inline(always)]
    pub fn rangebat(&self) -> RANGEBAT_R {
        RANGEBAT_R::new(((self.bits >> 8) & 0x03) as u8)
    }
    #[doc = "Bit 10 - Interrupt Enable for VBAT Range"]
    #[inline(always)]
    pub fn ienbat(&self) -> IENBAT_R {
        IENBAT_R::new(((self.bits >> 10) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Interrupt for VBAT"]
    #[inline(always)]
    pub fn vbat(&mut self) -> VBAT_W {
        VBAT_W { w: self }
    }
    #[doc = "Bit 1 - Enable Interrupt When VREG Undervoltage: Below 1V"]
    #[inline(always)]
    pub fn vregundr(&mut self) -> VREGUNDR_W {
        VREGUNDR_W { w: self }
    }
    #[doc = "Bit 2 - Enable Interrupt When VREG Overvoltage: Above 1.32V"]
    #[inline(always)]
    pub fn vregovr(&mut self) -> VREGOVR_W {
        VREGOVR_W { w: self }
    }
    #[doc = "Bits 8:9 - Battery Monitor Range"]
    #[inline(always)]
    pub fn rangebat(&mut self) -> RANGEBAT_W {
        RANGEBAT_W { w: self }
    }
    #[doc = "Bit 10 - Interrupt Enable for VBAT Range"]
    #[inline(always)]
    pub fn ienbat(&mut self) -> IENBAT_W {
        IENBAT_W { w: self }
    }
}
