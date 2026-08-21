#[doc = "Reader of register PWRUP"]
pub type R = crate::R<u16, super::PWRUP>;
#[doc = "Writer for register PWRUP"]
pub type W = crate::W<u16, super::PWRUP>;
#[doc = "Register PWRUP `reset()`'s with value 0x020e"]
impl crate::ResetValue for super::PWRUP {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x020e
    }
}
#[doc = "Reader of field `WAIT`"]
pub type WAIT_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `WAIT`"]
pub struct WAIT_W<'a> {
    w: &'a mut W,
}
impl<'a> WAIT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03ff) | ((value as u16) & 0x03ff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:9 - Program This with 526/PCLKDIVCNT"]
    #[inline(always)]
    pub fn wait(&self) -> WAIT_R {
        WAIT_R::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Program This with 526/PCLKDIVCNT"]
    #[inline(always)]
    pub fn wait(&mut self) -> WAIT_W {
        WAIT_W { w: self }
    }
}
