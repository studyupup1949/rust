#[doc = "Reader of register POR_SEC"]
pub type R = crate::R<u32, super::POR_SEC>;
#[doc = "Writer for register POR_SEC"]
pub type W = crate::W<u32, super::POR_SEC>;
#[doc = "Register POR_SEC `reset()`'s with value 0"]
impl crate::ResetValue for super::POR_SEC {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SECURE`"]
pub type SECURE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SECURE`"]
pub struct SECURE_W<'a> {
    w: &'a mut W,
}
impl<'a> SECURE_W<'a> {
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
    #[doc = "Bit 0 - Prevent Read/Write Access to User Space (Sticky When Set)"]
    #[inline(always)]
    pub fn secure(&self) -> SECURE_R {
        SECURE_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Prevent Read/Write Access to User Space (Sticky When Set)"]
    #[inline(always)]
    pub fn secure(&mut self) -> SECURE_W {
        SECURE_W { w: self }
    }
}
