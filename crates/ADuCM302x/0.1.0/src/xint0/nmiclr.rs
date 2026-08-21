#[doc = "Reader of register NMICLR"]
pub type R = crate::R<u32, super::NMICLR>;
#[doc = "Writer for register NMICLR"]
pub type W = crate::W<u32, super::NMICLR>;
#[doc = "Register NMICLR `reset()`'s with value 0"]
impl crate::ResetValue for super::NMICLR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CLR`"]
pub type CLR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CLR`"]
pub struct CLR_W<'a> {
    w: &'a mut W,
}
impl<'a> CLR_W<'a> {
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
    #[doc = "Bit 0 - NMI Clear"]
    #[inline(always)]
    pub fn clr(&self) -> CLR_R {
        CLR_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - NMI Clear"]
    #[inline(always)]
    pub fn clr(&mut self) -> CLR_W {
        CLR_W { w: self }
    }
}
