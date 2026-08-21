#[doc = "Reader of register SS1"]
pub type R = crate::R<u16, super::SS1>;
#[doc = "Writer for register SS1"]
pub type W = crate::W<u16, super::SS1>;
#[doc = "Register SS1 `reset()`'s with value 0x8000"]
impl crate::ResetValue for super::SS1 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x8000
    }
}
#[doc = "Reader of field `SS1`"]
pub type SS1_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `SS1`"]
pub struct SS1_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1(&self) -> SS1_R {
        SS1_R::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1(&mut self) -> SS1_W {
        SS1_W { w: self }
    }
}
