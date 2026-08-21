#[doc = "Reader of register ALERT"]
pub type R = crate::R<u16, super::ALERT>;
#[doc = "Writer for register ALERT"]
pub type W = crate::W<u16, super::ALERT>;
#[doc = "Register ALERT `reset()`'s with value 0"]
impl crate::ResetValue for super::ALERT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `HI0`"]
pub type HI0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HI0`"]
pub struct HI0_W<'a> {
    w: &'a mut W,
}
impl<'a> HI0_W<'a> {
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
#[doc = "Reader of field `LO0`"]
pub type LO0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LO0`"]
pub struct LO0_W<'a> {
    w: &'a mut W,
}
impl<'a> LO0_W<'a> {
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
#[doc = "Reader of field `HI1`"]
pub type HI1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HI1`"]
pub struct HI1_W<'a> {
    w: &'a mut W,
}
impl<'a> HI1_W<'a> {
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
#[doc = "Reader of field `LO1`"]
pub type LO1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LO1`"]
pub struct LO1_W<'a> {
    w: &'a mut W,
}
impl<'a> LO1_W<'a> {
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
#[doc = "Reader of field `HI2`"]
pub type HI2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HI2`"]
pub struct HI2_W<'a> {
    w: &'a mut W,
}
impl<'a> HI2_W<'a> {
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
#[doc = "Reader of field `LO2`"]
pub type LO2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LO2`"]
pub struct LO2_W<'a> {
    w: &'a mut W,
}
impl<'a> LO2_W<'a> {
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
#[doc = "Reader of field `HI3`"]
pub type HI3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HI3`"]
pub struct HI3_W<'a> {
    w: &'a mut W,
}
impl<'a> HI3_W<'a> {
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
#[doc = "Reader of field `LO3`"]
pub type LO3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LO3`"]
pub struct LO3_W<'a> {
    w: &'a mut W,
}
impl<'a> LO3_W<'a> {
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
    #[doc = "Bit 0 - Channel 0 High Alert Status"]
    #[inline(always)]
    pub fn hi0(&self) -> HI0_R {
        HI0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Channel 0 Low Alert Status"]
    #[inline(always)]
    pub fn lo0(&self) -> LO0_R {
        LO0_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Channel 1 High Alert Status"]
    #[inline(always)]
    pub fn hi1(&self) -> HI1_R {
        HI1_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Channel 1 Low Alert Status"]
    #[inline(always)]
    pub fn lo1(&self) -> LO1_R {
        LO1_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Channel 2 High Alert Status"]
    #[inline(always)]
    pub fn hi2(&self) -> HI2_R {
        HI2_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Channel 2 Low Alert Status"]
    #[inline(always)]
    pub fn lo2(&self) -> LO2_R {
        LO2_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Channel 3 High Alert Status"]
    #[inline(always)]
    pub fn hi3(&self) -> HI3_R {
        HI3_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Channel 3 Low Alert Status"]
    #[inline(always)]
    pub fn lo3(&self) -> LO3_R {
        LO3_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Channel 0 High Alert Status"]
    #[inline(always)]
    pub fn hi0(&mut self) -> HI0_W {
        HI0_W { w: self }
    }
    #[doc = "Bit 1 - Channel 0 Low Alert Status"]
    #[inline(always)]
    pub fn lo0(&mut self) -> LO0_W {
        LO0_W { w: self }
    }
    #[doc = "Bit 2 - Channel 1 High Alert Status"]
    #[inline(always)]
    pub fn hi1(&mut self) -> HI1_W {
        HI1_W { w: self }
    }
    #[doc = "Bit 3 - Channel 1 Low Alert Status"]
    #[inline(always)]
    pub fn lo1(&mut self) -> LO1_W {
        LO1_W { w: self }
    }
    #[doc = "Bit 4 - Channel 2 High Alert Status"]
    #[inline(always)]
    pub fn hi2(&mut self) -> HI2_W {
        HI2_W { w: self }
    }
    #[doc = "Bit 5 - Channel 2 Low Alert Status"]
    #[inline(always)]
    pub fn lo2(&mut self) -> LO2_W {
        LO2_W { w: self }
    }
    #[doc = "Bit 6 - Channel 3 High Alert Status"]
    #[inline(always)]
    pub fn hi3(&mut self) -> HI3_W {
        HI3_W { w: self }
    }
    #[doc = "Bit 7 - Channel 3 Low Alert Status"]
    #[inline(always)]
    pub fn lo3(&mut self) -> LO3_W {
        LO3_W { w: self }
    }
}
