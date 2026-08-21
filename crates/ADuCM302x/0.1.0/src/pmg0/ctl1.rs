#[doc = "Reader of register CTL1"]
pub type R = crate::R<u32, super::CTL1>;
#[doc = "Writer for register CTL1"]
pub type W = crate::W<u32, super::CTL1>;
#[doc = "Register CTL1 `reset()`'s with value 0x00a0_0000"]
impl crate::ResetValue for super::CTL1 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x00a0_0000
    }
}
#[doc = "Reader of field `HPBUCKEN`"]
pub type HPBUCKEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HPBUCKEN`"]
pub struct HPBUCKEN_W<'a> {
    w: &'a mut W,
}
impl<'a> HPBUCKEN_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Enable HP Buck"]
    #[inline(always)]
    pub fn hpbucken(&self) -> HPBUCKEN_R {
        HPBUCKEN_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable HP Buck"]
    #[inline(always)]
    pub fn hpbucken(&mut self) -> HPBUCKEN_W {
        HPBUCKEN_W { w: self }
    }
}
