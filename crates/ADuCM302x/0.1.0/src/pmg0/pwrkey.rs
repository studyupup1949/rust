#[doc = "Writer for register PWRKEY"]
pub type W = crate::W<u32, super::PWRKEY>;
#[doc = "Register PWRKEY `reset()`'s with value 0"]
impl crate::ResetValue for super::PWRKEY {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u32) & 0xffff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:15 - Power Control Key Register"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
