#[doc = "Reader of register LEN"]
pub type R = crate::R<u16, super::LEN>;
#[doc = "Writer for register LEN"]
pub type W = crate::W<u16, super::LEN>;
#[doc = "Register LEN `reset()`'s with value 0x3400"]
impl crate::ResetValue for super::LEN {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x3400
    }
}
#[doc = "Reader of field `RELOAD`"]
pub type RELOAD_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `RELOAD`"]
pub struct RELOAD_W<'a> {
    w: &'a mut W,
}
impl<'a> RELOAD_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0fff) | ((value as u16) & 0x0fff);
        self.w
    }
}
#[doc = "Reader of field `PRESCALE`"]
pub type PRESCALE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PRESCALE`"]
pub struct PRESCALE_W<'a> {
    w: &'a mut W,
}
impl<'a> PRESCALE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 12)) | (((value as u16) & 0x0f) << 12);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:11 - Reload Value for the Sample Counter"]
    #[inline(always)]
    pub fn reload(&self) -> RELOAD_R {
        RELOAD_R::new((self.bits & 0x0fff) as u16)
    }
    #[doc = "Bits 12:15 - Prescaler for the Sample Counter"]
    #[inline(always)]
    pub fn prescale(&self) -> PRESCALE_R {
        PRESCALE_R::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:11 - Reload Value for the Sample Counter"]
    #[inline(always)]
    pub fn reload(&mut self) -> RELOAD_W {
        RELOAD_W { w: self }
    }
    #[doc = "Bits 12:15 - Prescaler for the Sample Counter"]
    #[inline(always)]
    pub fn prescale(&mut self) -> PRESCALE_W {
        PRESCALE_W { w: self }
    }
}
