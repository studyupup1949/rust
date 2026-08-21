#[doc = "Writer for register PRI_CLR"]
pub type W = crate::W<u32, super::PRI_CLR>;
#[doc = "Register PRI_CLR `reset()`'s with value 0"]
impl crate::ResetValue for super::PRI_CLR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `CHPRICLR`"]
pub struct CHPRICLR_W<'a> {
    w: &'a mut W,
}
impl<'a> CHPRICLR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01ff_ffff) | ((value as u32) & 0x01ff_ffff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:24 - Configure Channel for Default Priority Level"]
    #[inline(always)]
    pub fn chpriclr(&mut self) -> CHPRICLR_W {
        CHPRICLR_W { w: self }
    }
}
