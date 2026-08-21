#[doc = "Reader of register WRPROT"]
pub type R = crate::R<u32, super::WRPROT>;
#[doc = "Writer for register WRPROT"]
pub type W = crate::W<u32, super::WRPROT>;
#[doc = "Register WRPROT `reset()`'s with value 0xffff_ffff"]
impl crate::ResetValue for super::WRPROT {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0xffff_ffff
    }
}
#[doc = "Reader of field `WORD`"]
pub type WORD_R = crate::R<u32, u32>;
#[doc = "Write proxy for field `WORD`"]
pub struct WORD_W<'a> {
    w: &'a mut W,
}
impl<'a> WORD_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff_ffff) | ((value as u32) & 0xffff_ffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:31 - Write Protect"]
    #[inline(always)]
    pub fn word(&self) -> WORD_R {
        WORD_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
impl W {
    #[doc = "Bits 0:31 - Write Protect"]
    #[inline(always)]
    pub fn word(&mut self) -> WORD_W {
        WORD_W { w: self }
    }
}
