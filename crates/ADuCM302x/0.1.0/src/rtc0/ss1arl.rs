#[doc = "Reader of register SS1ARL"]
pub type R = crate::R<u16, super::SS1ARL>;
#[doc = "Writer for register SS1ARL"]
pub type W = crate::W<u16, super::SS1ARL>;
#[doc = "Register SS1ARL `reset()`'s with value 0"]
impl crate::ResetValue for super::SS1ARL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SS1ARL`"]
pub type SS1ARL_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `SS1ARL`"]
pub struct SS1ARL_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1ARL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - Auto-Reload Value When SensorStrobe Match Occurs"]
    #[inline(always)]
    pub fn ss1arl(&self) -> SS1ARL_R {
        SS1ARL_R::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Auto-Reload Value When SensorStrobe Match Occurs"]
    #[inline(always)]
    pub fn ss1arl(&mut self) -> SS1ARL_W {
        SS1ARL_W { w: self }
    }
}
