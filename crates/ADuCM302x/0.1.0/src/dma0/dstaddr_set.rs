#[doc = "Reader of register DSTADDR_SET"]
pub type R = crate::R<u32, super::DSTADDR_SET>;
#[doc = "Writer for register DSTADDR_SET"]
pub type W = crate::W<u32, super::DSTADDR_SET>;
#[doc = "Register DSTADDR_SET `reset()`'s with value 0"]
impl crate::ResetValue for super::DSTADDR_SET {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CHAN`"]
pub type CHAN_R = crate::R<u32, u32>;
#[doc = "Write proxy for field `CHAN`"]
pub struct CHAN_W<'a> {
    w: &'a mut W,
}
impl<'a> CHAN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01ff_ffff) | ((value as u32) & 0x01ff_ffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:24 - Destination Address Decrement Status"]
    #[inline(always)]
    pub fn chan(&self) -> CHAN_R {
        CHAN_R::new((self.bits & 0x01ff_ffff) as u32)
    }
}
impl W {
    #[doc = "Bits 0:24 - Destination Address Decrement Status"]
    #[inline(always)]
    pub fn chan(&mut self) -> CHAN_W {
        CHAN_W { w: self }
    }
}
