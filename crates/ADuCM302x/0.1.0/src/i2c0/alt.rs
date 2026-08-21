#[doc = "Reader of register ALT"]
pub type R = crate::R<u16, super::ALT>;
#[doc = "Writer for register ALT"]
pub type W = crate::W<u16, super::ALT>;
#[doc = "Register ALT `reset()`'s with value 0"]
impl crate::ResetValue for super::ALT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `ID`"]
pub type ID_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `ID`"]
pub struct ID_W<'a> {
    w: &'a mut W,
}
impl<'a> ID_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Slave Alt"]
    #[inline(always)]
    pub fn id(&self) -> ID_R {
        ID_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Slave Alt"]
    #[inline(always)]
    pub fn id(&mut self) -> ID_W {
        ID_W { w: self }
    }
}
