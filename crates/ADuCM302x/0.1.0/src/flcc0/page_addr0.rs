#[doc = "Reader of register PAGE_ADDR0"]
pub type R = crate::R<u32, super::PAGE_ADDR0>;
#[doc = "Writer for register PAGE_ADDR0"]
pub type W = crate::W<u32, super::PAGE_ADDR0>;
#[doc = "Register PAGE_ADDR0 `reset()`'s with value 0"]
impl crate::ResetValue for super::PAGE_ADDR0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
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
        self.w.bits = (self.w.bits & !(0x01ff << 10)) | (((value as u32) & 0x01ff) << 10);
        self.w
    }
}
impl R {
    #[doc = "Bits 10:18 - Lower Address Bits of the Page Address"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new(((self.bits >> 10) & 0x01ff) as u16)
    }
}
impl W {
    #[doc = "Bits 10:18 - Lower Address Bits of the Page Address"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
