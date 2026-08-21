#[doc = "Reader of register TRM"]
pub type R = crate::R<u16, super::TRM>;
#[doc = "Writer for register TRM"]
pub type W = crate::W<u16, super::TRM>;
#[doc = "Register TRM `reset()`'s with value 0x0398"]
impl crate::ResetValue for super::TRM {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0398
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x07) | ((value as u16) & 0x07);
        self.w
    }
}
#[doc = "Reader of field `ADD`"]
pub type ADD_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ADD`"]
pub struct ADD_W<'a> {
    w: &'a mut W,
}
impl<'a> ADD_W<'a> {
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
#[doc = "Reader of field `IVL`"]
pub type IVL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IVL`"]
pub struct IVL_W<'a> {
    w: &'a mut W,
}
impl<'a> IVL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 4)) | (((value as u16) & 0x03) << 4);
        self.w
    }
}
#[doc = "Reader of field `IVL2EXPMIN`"]
pub type IVL2EXPMIN_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IVL2EXPMIN`"]
pub struct IVL2EXPMIN_W<'a> {
    w: &'a mut W,
}
impl<'a> IVL2EXPMIN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 6)) | (((value as u16) & 0x0f) << 6);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:2 - Trim Value in Prescaled RTC Time Units to Be Added or Subtracted from the RTC Count at the End of a Periodic Interval Selected by TRM:TRMIVL"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x07) as u8)
    }
    #[doc = "Bit 3 - Trim Polarity"]
    #[inline(always)]
    pub fn add(&self) -> ADD_R {
        ADD_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:5 - Trim Interval in Prescaled RTC Time Units"]
    #[inline(always)]
    pub fn ivl(&self) -> IVL_R {
        IVL_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 6:9 - Minimum Power-of-two Interval of Prescaled RTC Time Units Which TRM:TRMIVL TRMIVL Can Select"]
    #[inline(always)]
    pub fn ivl2expmin(&self) -> IVL2EXPMIN_R {
        IVL2EXPMIN_R::new(((self.bits >> 6) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Trim Value in Prescaled RTC Time Units to Be Added or Subtracted from the RTC Count at the End of a Periodic Interval Selected by TRM:TRMIVL"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
    #[doc = "Bit 3 - Trim Polarity"]
    #[inline(always)]
    pub fn add(&mut self) -> ADD_W {
        ADD_W { w: self }
    }
    #[doc = "Bits 4:5 - Trim Interval in Prescaled RTC Time Units"]
    #[inline(always)]
    pub fn ivl(&mut self) -> IVL_W {
        IVL_W { w: self }
    }
    #[doc = "Bits 6:9 - Minimum Power-of-two Interval of Prescaled RTC Time Units Which TRM:TRMIVL TRMIVL Can Select"]
    #[inline(always)]
    pub fn ivl2expmin(&mut self) -> IVL2EXPMIN_W {
        IVL2EXPMIN_W { w: self }
    }
}
