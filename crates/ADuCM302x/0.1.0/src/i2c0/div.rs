#[doc = "Reader of register DIV"]
pub type R = crate::R<u16, super::DIV>;
#[doc = "Writer for register DIV"]
pub type W = crate::W<u16, super::DIV>;
#[doc = "Register DIV `reset()`'s with value 0x1f1f"]
impl crate::ResetValue for super::DIV {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x1f1f
    }
}
#[doc = "Reader of field `LOW`"]
pub type LOW_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `LOW`"]
pub struct LOW_W<'a> {
    w: &'a mut W,
}
impl<'a> LOW_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `HIGH`"]
pub type HIGH_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `HIGH`"]
pub struct HIGH_W<'a> {
    w: &'a mut W,
}
impl<'a> HIGH_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 8)) | (((value as u16) & 0xff) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Serial Clock Low Time"]
    #[inline(always)]
    pub fn low(&self) -> LOW_R {
        LOW_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Serial Clock High Time"]
    #[inline(always)]
    pub fn high(&self) -> HIGH_R {
        HIGH_R::new(((self.bits >> 8) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Serial Clock Low Time"]
    #[inline(always)]
    pub fn low(&mut self) -> LOW_W {
        LOW_W { w: self }
    }
    #[doc = "Bits 8:15 - Serial Clock High Time"]
    #[inline(always)]
    pub fn high(&mut self) -> HIGH_W {
        HIGH_W { w: self }
    }
}
