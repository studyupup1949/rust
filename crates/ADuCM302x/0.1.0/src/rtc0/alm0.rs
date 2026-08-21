#[doc = "Reader of register ALM0"]
pub type R = crate::R<u16, super::ALM0>;
#[doc = "Writer for register ALM0"]
pub type W = crate::W<u16, super::ALM0>;
#[doc = "Register ALM0 `reset()`'s with value 0xffff"]
impl crate::ResetValue for super::ALM0 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0xffff
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - Lower 16 Prescaled (i.e. Non-Fractional) Bits of the RTC Alarm Target Time"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Lower 16 Prescaled (i.e. Non-Fractional) Bits of the RTC Alarm Target Time"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
