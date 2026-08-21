#[doc = "Reader of register DIV_A"]
pub type R = crate::R<u32, super::DIV_A>;
#[doc = "Writer for register DIV_A"]
pub type W = crate::W<u32, super::DIV_A>;
#[doc = "Register DIV_A `reset()`'s with value 0"]
impl crate::ResetValue for super::DIV_A {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CLKDIV`"]
pub type CLKDIV_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `CLKDIV`"]
pub struct CLKDIV_W<'a> {
    w: &'a mut W,
}
impl<'a> CLKDIV_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u32) & 0xffff);
        self.w
    }
}
#[doc = "Reader of field `FSDIV`"]
pub type FSDIV_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FSDIV`"]
pub struct FSDIV_W<'a> {
    w: &'a mut W,
}
impl<'a> FSDIV_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 16)) | (((value as u32) & 0xff) << 16);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - Clock Divisor"]
    #[inline(always)]
    pub fn clkdiv(&self) -> CLKDIV_R {
        CLKDIV_R::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:23 - Frame Sync Divisor"]
    #[inline(always)]
    pub fn fsdiv(&self) -> FSDIV_R {
        FSDIV_R::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:15 - Clock Divisor"]
    #[inline(always)]
    pub fn clkdiv(&mut self) -> CLKDIV_W {
        CLKDIV_W { w: self }
    }
    #[doc = "Bits 16:23 - Frame Sync Divisor"]
    #[inline(always)]
    pub fn fsdiv(&mut self) -> FSDIV_W {
        FSDIV_W { w: self }
    }
}
