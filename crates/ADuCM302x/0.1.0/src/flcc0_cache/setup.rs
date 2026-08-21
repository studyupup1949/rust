#[doc = "Reader of register SETUP"]
pub type R = crate::R<u32, super::SETUP>;
#[doc = "Writer for register SETUP"]
pub type W = crate::W<u32, super::SETUP>;
#[doc = "Register SETUP `reset()`'s with value 0"]
impl crate::ResetValue for super::SETUP {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `ICEN`"]
pub type ICEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ICEN`"]
pub struct ICEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ICEN_W<'a> {
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
    #[doc = "Bit 0 - I-Cache Enable"]
    #[inline(always)]
    pub fn icen(&self) -> ICEN_R {
        ICEN_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - I-Cache Enable"]
    #[inline(always)]
    pub fn icen(&mut self) -> ICEN_W {
        ICEN_W { w: self }
    }
}
