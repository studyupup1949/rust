#[doc = "Reader of register CFG1"]
pub type R = crate::R<u16, super::CFG1>;
#[doc = "Writer for register CFG1"]
pub type W = crate::W<u16, super::CFG1>;
#[doc = "Register CFG1 `reset()`'s with value 0x0400"]
impl crate::ResetValue for super::CFG1 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0400
    }
}
#[doc = "Reader of field `RBUFLP`"]
pub type RBUFLP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RBUFLP`"]
pub struct RBUFLP_W<'a> {
    w: &'a mut W,
}
impl<'a> RBUFLP_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Enable Low Power Mode for Reference Buffer"]
    #[inline(always)]
    pub fn rbuflp(&self) -> RBUFLP_R {
        RBUFLP_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Low Power Mode for Reference Buffer"]
    #[inline(always)]
    pub fn rbuflp(&mut self) -> RBUFLP_W {
        RBUFLP_W { w: self }
    }
}
