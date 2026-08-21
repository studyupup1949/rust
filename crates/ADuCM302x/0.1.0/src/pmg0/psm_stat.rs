#[doc = "Reader of register PSM_STAT"]
pub type R = crate::R<u32, super::PSM_STAT>;
#[doc = "Writer for register PSM_STAT"]
pub type W = crate::W<u32, super::PSM_STAT>;
#[doc = "Register PSM_STAT `reset()`'s with value 0"]
impl crate::ResetValue for super::PSM_STAT {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `VBATUNDR`"]
pub type VBATUNDR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `VBATUNDR`"]
pub struct VBATUNDR_W<'a> {
    w: &'a mut W,
}
impl<'a> VBATUNDR_W<'a> {
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
#[doc = "Reader of field `WICENACK`"]
pub type WICENACK_R = crate::R<bool, bool>;
#[doc = "Reader of field `RANGE1`"]
pub type RANGE1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RANGE1`"]
pub struct RANGE1_W<'a> {
    w: &'a mut W,
}
impl<'a> RANGE1_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u32) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `RANGE2`"]
pub type RANGE2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RANGE2`"]
pub struct RANGE2_W<'a> {
    w: &'a mut W,
}
impl<'a> RANGE2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u32) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `RANGE3`"]
pub type RANGE3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RANGE3`"]
pub struct RANGE3_W<'a> {
    w: &'a mut W,
}
impl<'a> RANGE3_W<'a> {
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
#[doc = "VBAT Range1 (> 2.75v)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RORANGE1_A {
    #[doc = "0: VBAT not in the range specified"]
    BATSTAT1 = 0,
    #[doc = "1: VBAT in the range specified"]
    BATSTAT0 = 1,
}
impl From<RORANGE1_A> for bool {
    #[inline(always)]
    fn from(variant: RORANGE1_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `RORANGE1`"]
pub type RORANGE1_R = crate::R<bool, RORANGE1_A>;
impl RORANGE1_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> RORANGE1_A {
        match self.bits {
            false => RORANGE1_A::BATSTAT1,
            true => RORANGE1_A::BATSTAT0,
        }
    }
    #[doc = "Checks if the value of the field is `BATSTAT1`"]
    #[inline(always)]
    pub fn is_batstat1(&self) -> bool {
        *self == RORANGE1_A::BATSTAT1
    }
    #[doc = "Checks if the value of the field is `BATSTAT0`"]
    #[inline(always)]
    pub fn is_batstat0(&self) -> bool {
        *self == RORANGE1_A::BATSTAT0
    }
}
#[doc = "Reader of field `RORANGE2`"]
pub type RORANGE2_R = crate::R<bool, bool>;
#[doc = "Reader of field `RORANGE3`"]
pub type RORANGE3_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Status Bit Indicating an Alarm That Battery is Below 1.8V"]
    #[inline(always)]
    pub fn vbatundr(&self) -> VBATUNDR_R {
        VBATUNDR_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Status Bit for Alarm Indicating VREG is Below 1V"]
    #[inline(always)]
    pub fn vregundr(&self) -> VREGUNDR_R {
        VREGUNDR_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Status Bit for Alarm Indicating Overvoltage for VREG"]
    #[inline(always)]
    pub fn vregovr(&self) -> VREGOVR_R {
        VREGOVR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 7 - WIC Enable Acknowledge from Cortex"]
    #[inline(always)]
    pub fn wicenack(&self) -> WICENACK_R {
        WICENACK_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - VBAT Range1 (> 2.75v)"]
    #[inline(always)]
    pub fn range1(&self) -> RANGE1_R {
        RANGE1_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - VBAT Range2 (2.75v - 2.3v)"]
    #[inline(always)]
    pub fn range2(&self) -> RANGE2_R {
        RANGE2_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - VBAT Range3 (2.3v - 1.6v)"]
    #[inline(always)]
    pub fn range3(&self) -> RANGE3_R {
        RANGE3_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 13 - VBAT Range1 (> 2.75v)"]
    #[inline(always)]
    pub fn rorange1(&self) -> RORANGE1_R {
        RORANGE1_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - VBAT Range2 (2.75v - 2.3v)"]
    #[inline(always)]
    pub fn rorange2(&self) -> RORANGE2_R {
        RORANGE2_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - VBAT Range3 (2.3v - 1.6v)"]
    #[inline(always)]
    pub fn rorange3(&self) -> RORANGE3_R {
        RORANGE3_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Status Bit Indicating an Alarm That Battery is Below 1.8V"]
    #[inline(always)]
    pub fn vbatundr(&mut self) -> VBATUNDR_W {
        VBATUNDR_W { w: self }
    }
    #[doc = "Bit 1 - Status Bit for Alarm Indicating VREG is Below 1V"]
    #[inline(always)]
    pub fn vregundr(&mut self) -> VREGUNDR_W {
        VREGUNDR_W { w: self }
    }
    #[doc = "Bit 2 - Status Bit for Alarm Indicating Overvoltage for VREG"]
    #[inline(always)]
    pub fn vregovr(&mut self) -> VREGOVR_W {
        VREGOVR_W { w: self }
    }
    #[doc = "Bit 8 - VBAT Range1 (> 2.75v)"]
    #[inline(always)]
    pub fn range1(&mut self) -> RANGE1_W {
        RANGE1_W { w: self }
    }
    #[doc = "Bit 9 - VBAT Range2 (2.75v - 2.3v)"]
    #[inline(always)]
    pub fn range2(&mut self) -> RANGE2_W {
        RANGE2_W { w: self }
    }
    #[doc = "Bit 10 - VBAT Range3 (2.3v - 1.6v)"]
    #[inline(always)]
    pub fn range3(&mut self) -> RANGE3_W {
        RANGE3_W { w: self }
    }
}
