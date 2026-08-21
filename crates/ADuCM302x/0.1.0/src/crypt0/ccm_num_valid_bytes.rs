#[doc = "Reader of register CCM_NUM_VALID_BYTES"]
pub type R = crate::R<u32, super::CCM_NUM_VALID_BYTES>;
#[doc = "Writer for register CCM_NUM_VALID_BYTES"]
pub type W = crate::W<u32, super::CCM_NUM_VALID_BYTES>;
#[doc = "Register CCM_NUM_VALID_BYTES `reset()`'s with value 0"]
impl crate::ResetValue for super::CCM_NUM_VALID_BYTES {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `NUM_VALID_BYTES`"]
pub type NUM_VALID_BYTES_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `NUM_VALID_BYTES`"]
pub struct NUM_VALID_BYTES_W<'a> {
    w: &'a mut W,
}
impl<'a> NUM_VALID_BYTES_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u32) & 0x0f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:3 - Number of Valid Bytes in CCM Last Data"]
    #[inline(always)]
    pub fn num_valid_bytes(&self) -> NUM_VALID_BYTES_R {
        NUM_VALID_BYTES_R::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Number of Valid Bytes in CCM Last Data"]
    #[inline(always)]
    pub fn num_valid_bytes(&mut self) -> NUM_VALID_BYTES_W {
        NUM_VALID_BYTES_W { w: self }
    }
}
