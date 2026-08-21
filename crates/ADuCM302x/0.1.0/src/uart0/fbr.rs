#[doc = "Reader of register FBR"]
pub type R = crate::R<u16, super::FBR>;
#[doc = "Writer for register FBR"]
pub type W = crate::W<u16, super::FBR>;
#[doc = "Register FBR `reset()`'s with value 0"]
impl crate::ResetValue for super::FBR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `DIVN`"]
pub type DIVN_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `DIVN`"]
pub struct DIVN_W<'a> {
    w: &'a mut W,
}
impl<'a> DIVN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x07ff) | ((value as u16) & 0x07ff);
        self.w
    }
}
#[doc = "Reader of field `DIVM`"]
pub type DIVM_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DIVM`"]
pub struct DIVM_W<'a> {
    w: &'a mut W,
}
impl<'a> DIVM_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 11)) | (((value as u16) & 0x03) << 11);
        self.w
    }
}
#[doc = "Reader of field `FBEN`"]
pub type FBEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FBEN`"]
pub struct FBEN_W<'a> {
    w: &'a mut W,
}
impl<'a> FBEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u16) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:10 - Fractional Baud Rate N Divide Bits 0 to 2047"]
    #[inline(always)]
    pub fn divn(&self) -> DIVN_R {
        DIVN_R::new((self.bits & 0x07ff) as u16)
    }
    #[doc = "Bits 11:12 - Fractional Baud Rate M Divide Bits 1 to 3"]
    #[inline(always)]
    pub fn divm(&self) -> DIVM_R {
        DIVM_R::new(((self.bits >> 11) & 0x03) as u8)
    }
    #[doc = "Bit 15 - Fractional Baud Rate Generator Enable"]
    #[inline(always)]
    pub fn fben(&self) -> FBEN_R {
        FBEN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:10 - Fractional Baud Rate N Divide Bits 0 to 2047"]
    #[inline(always)]
    pub fn divn(&mut self) -> DIVN_W {
        DIVN_W { w: self }
    }
    #[doc = "Bits 11:12 - Fractional Baud Rate M Divide Bits 1 to 3"]
    #[inline(always)]
    pub fn divm(&mut self) -> DIVM_W {
        DIVM_W { w: self }
    }
    #[doc = "Bit 15 - Fractional Baud Rate Generator Enable"]
    #[inline(always)]
    pub fn fben(&mut self) -> FBEN_W {
        FBEN_W { w: self }
    }
}
