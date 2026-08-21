#[doc = "Reader of register SHA_LAST_WORD"]
pub type R = crate::R<u32, super::SHA_LAST_WORD>;
#[doc = "Writer for register SHA_LAST_WORD"]
pub type W = crate::W<u32, super::SHA_LAST_WORD>;
#[doc = "Register SHA_LAST_WORD `reset()`'s with value 0"]
impl crate::ResetValue for super::SHA_LAST_WORD {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `O_Last_Word`"]
pub type O_LAST_WORD_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `O_Last_Word`"]
pub struct O_LAST_WORD_W<'a> {
    w: &'a mut W,
}
impl<'a> O_LAST_WORD_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `O_Bits_Valid`"]
pub type O_BITS_VALID_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `O_Bits_Valid`"]
pub struct O_BITS_VALID_W<'a> {
    w: &'a mut W,
}
impl<'a> O_BITS_VALID_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x1f << 1)) | (((value as u32) & 0x1f) << 1);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Last SHA Input Word"]
    #[inline(always)]
    pub fn o_last_word(&self) -> O_LAST_WORD_R {
        O_LAST_WORD_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bits 1:5 - Bits Valid in SHA Last Word Input"]
    #[inline(always)]
    pub fn o_bits_valid(&self) -> O_BITS_VALID_R {
        O_BITS_VALID_R::new(((self.bits >> 1) & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Last SHA Input Word"]
    #[inline(always)]
    pub fn o_last_word(&mut self) -> O_LAST_WORD_W {
        O_LAST_WORD_W { w: self }
    }
    #[doc = "Bits 1:5 - Bits Valid in SHA Last Word Input"]
    #[inline(always)]
    pub fn o_bits_valid(&mut self) -> O_BITS_VALID_W {
        O_BITS_VALID_W { w: self }
    }
}
