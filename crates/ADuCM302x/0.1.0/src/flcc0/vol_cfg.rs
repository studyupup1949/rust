#[doc = "Reader of register VOL_CFG"]
pub type R = crate::R<u32, super::VOL_CFG>;
#[doc = "Writer for register VOL_CFG"]
pub type W = crate::W<u32, super::VOL_CFG>;
#[doc = "Register VOL_CFG `reset()`'s with value 0x01"]
impl crate::ResetValue for super::VOL_CFG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x01
    }
}
#[doc = "Reader of field `INFO_REMAP`"]
pub type INFO_REMAP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INFO_REMAP`"]
pub struct INFO_REMAP_W<'a> {
    w: &'a mut W,
}
impl<'a> INFO_REMAP_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Alias the Info Space to the Base Address of User Space"]
    #[inline(always)]
    pub fn info_remap(&self) -> INFO_REMAP_R {
        INFO_REMAP_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Alias the Info Space to the Base Address of User Space"]
    #[inline(always)]
    pub fn info_remap(&mut self) -> INFO_REMAP_W {
        INFO_REMAP_W { w: self }
    }
}
