#[doc = "Reader of register SCR"]
pub type R = crate::R<u16, super::SCR>;
#[doc = "Writer for register SCR"]
pub type W = crate::W<u16, super::SCR>;
#[doc = "Register SCR `reset()`'s with value 0"]
impl crate::ResetValue for super::SCR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SCR`"]
pub type SCR_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SCR`"]
pub struct SCR_W<'a> {
    w: &'a mut W,
}
impl<'a> SCR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Scratch"]
    #[inline(always)]
    pub fn scr(&self) -> SCR_R {
        SCR_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Scratch"]
    #[inline(always)]
    pub fn scr(&mut self) -> SCR_W {
        SCR_W { w: self }
    }
}
