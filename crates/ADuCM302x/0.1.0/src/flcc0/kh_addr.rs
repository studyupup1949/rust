#[doc = "Reader of register KH_ADDR"]
pub type R = crate::R<u32, super::KH_ADDR>;
#[doc = "Writer for register KH_ADDR"]
pub type W = crate::W<u32, super::KH_ADDR>;
#[doc = "Register KH_ADDR `reset()`'s with value 0"]
impl crate::ResetValue for super::KH_ADDR {
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
        self.w.bits = (self.w.bits & !(0xffff << 3)) | (((value as u32) & 0xffff) << 3);
        self.w
    }
}
impl R {
    #[doc = "Bits 3:18 - Key Hole Address"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new(((self.bits >> 3) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 3:18 - Key Hole Address"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
