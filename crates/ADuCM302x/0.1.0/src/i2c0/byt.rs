#[doc = "Reader of register BYT"]
pub type R = crate::R<u16, super::BYT>;
#[doc = "Writer for register BYT"]
pub type W = crate::W<u16, super::BYT>;
#[doc = "Register BYT `reset()`'s with value 0"]
impl crate::ResetValue for super::BYT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SBYTE`"]
pub type SBYTE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SBYTE`"]
pub struct SBYTE_W<'a> {
    w: &'a mut W,
}
impl<'a> SBYTE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Start Byte"]
    #[inline(always)]
    pub fn sbyte(&self) -> SBYTE_R {
        SBYTE_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Start Byte"]
    #[inline(always)]
    pub fn sbyte(&mut self) -> SBYTE_W {
        SBYTE_W { w: self }
    }
}
