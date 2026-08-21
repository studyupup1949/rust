#[doc = "Reader of register TIME_PARAM1"]
pub type R = crate::R<u32, super::TIME_PARAM1>;
#[doc = "Writer for register TIME_PARAM1"]
pub type W = crate::W<u32, super::TIME_PARAM1>;
#[doc = "Register TIME_PARAM1 `reset()`'s with value 0x04"]
impl crate::ResetValue for super::TIME_PARAM1 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x04
    }
}
#[doc = "Reader of field `TWK`"]
pub type TWK_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TWK`"]
pub struct TWK_W<'a> {
    w: &'a mut W,
}
impl<'a> TWK_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u32) & 0x0f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:3 - Wakeup Time"]
    #[inline(always)]
    pub fn twk(&self) -> TWK_R {
        TWK_R::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Wakeup Time"]
    #[inline(always)]
    pub fn twk(&mut self) -> TWK_W {
        TWK_W { w: self }
    }
}
