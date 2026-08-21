#[doc = "Reader of register STAT"]
pub type R = crate::R<u32, super::STAT>;
#[doc = "Writer for register STAT"]
pub type W = crate::W<u32, super::STAT>;
#[doc = "Register STAT `reset()`'s with value 0x01"]
impl crate::ResetValue for super::STAT {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x01
    }
}
#[doc = "Reader of field `INRDY`"]
pub type INRDY_R = crate::R<bool, bool>;
#[doc = "Reader of field `OUTRDY`"]
pub type OUTRDY_R = crate::R<bool, bool>;
#[doc = "Reader of field `INOVR`"]
pub type INOVR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INOVR`"]
pub struct INOVR_W<'a> {
    w: &'a mut W,
}
impl<'a> INOVR_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `SHADONE`"]
pub type SHADONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SHADONE`"]
pub struct SHADONE_W<'a> {
    w: &'a mut W,
}
impl<'a> SHADONE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `SHABUSY`"]
pub type SHABUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `INWORDS`"]
pub type INWORDS_R = crate::R<u8, u8>;
#[doc = "Reader of field `OUTWORDS`"]
pub type OUTWORDS_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - Input Buffer Status"]
    #[inline(always)]
    pub fn inrdy(&self) -> INRDY_R {
        INRDY_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Output Data Ready"]
    #[inline(always)]
    pub fn outrdy(&self) -> OUTRDY_R {
        OUTRDY_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Overflow in the Input Buffer"]
    #[inline(always)]
    pub fn inovr(&self) -> INOVR_R {
        INOVR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 5 - SHA Computation Complete"]
    #[inline(always)]
    pub fn shadone(&self) -> SHADONE_R {
        SHADONE_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - SHA Busy. in Computation"]
    #[inline(always)]
    pub fn shabusy(&self) -> SHABUSY_R {
        SHABUSY_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bits 7:9 - Number of Words in the Input Buffer"]
    #[inline(always)]
    pub fn inwords(&self) -> INWORDS_R {
        INWORDS_R::new(((self.bits >> 7) & 0x07) as u8)
    }
    #[doc = "Bits 10:12 - Number of Words in the Output Buffer"]
    #[inline(always)]
    pub fn outwords(&self) -> OUTWORDS_R {
        OUTWORDS_R::new(((self.bits >> 10) & 0x07) as u8)
    }
}
impl W {
    #[doc = "Bit 2 - Overflow in the Input Buffer"]
    #[inline(always)]
    pub fn inovr(&mut self) -> INOVR_W {
        INOVR_W { w: self }
    }
    #[doc = "Bit 5 - SHA Computation Complete"]
    #[inline(always)]
    pub fn shadone(&mut self) -> SHADONE_W {
        SHADONE_W { w: self }
    }
}
