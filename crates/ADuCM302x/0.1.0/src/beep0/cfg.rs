#[doc = "Reader of register CFG"]
pub type R = crate::R<u16, super::CFG>;
#[doc = "Writer for register CFG"]
pub type W = crate::W<u16, super::CFG>;
#[doc = "Register CFG `reset()`'s with value 0"]
impl crate::ResetValue for super::CFG {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SEQREPEAT`"]
pub type SEQREPEAT_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SEQREPEAT`"]
pub struct SEQREPEAT_W<'a> {
    w: &'a mut W,
}
impl<'a> SEQREPEAT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Write proxy for field `EN`"]
pub struct EN_W<'a> {
    w: &'a mut W,
}
impl<'a> EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `ASTARTIRQ`"]
pub type ASTARTIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ASTARTIRQ`"]
pub struct ASTARTIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> ASTARTIRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u16) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `AENDIRQ`"]
pub type AENDIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AENDIRQ`"]
pub struct AENDIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> AENDIRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u16) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `BSTARTIRQ`"]
pub type BSTARTIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BSTARTIRQ`"]
pub struct BSTARTIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> BSTARTIRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u16) & 0x01) << 12);
        self.w
    }
}
#[doc = "Reader of field `BENDIRQ`"]
pub type BENDIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BENDIRQ`"]
pub struct BENDIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> BENDIRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u16) & 0x01) << 13);
        self.w
    }
}
#[doc = "Reader of field `SEQNEARENDIRQ`"]
pub type SEQNEARENDIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SEQNEARENDIRQ`"]
pub struct SEQNEARENDIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> SEQNEARENDIRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u16) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `SEQATENDIRQ`"]
pub type SEQATENDIRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SEQATENDIRQ`"]
pub struct SEQATENDIRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> SEQATENDIRQ_W<'a> {
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
    #[doc = "Bits 0:7 - Beeper Sequence Repeat Value"]
    #[inline(always)]
    pub fn seqrepeat(&self) -> SEQREPEAT_R {
        SEQREPEAT_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 10 - Tone A Start IRQ"]
    #[inline(always)]
    pub fn astartirq(&self) -> ASTARTIRQ_R {
        ASTARTIRQ_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Tone A End IRQ"]
    #[inline(always)]
    pub fn aendirq(&self) -> AENDIRQ_R {
        AENDIRQ_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Tone B Start IRQ"]
    #[inline(always)]
    pub fn bstartirq(&self) -> BSTARTIRQ_R {
        BSTARTIRQ_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Tone B End IRQ"]
    #[inline(always)]
    pub fn bendirq(&self) -> BENDIRQ_R {
        BENDIRQ_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Sequence 1 Cycle from End IRQ"]
    #[inline(always)]
    pub fn seqnearendirq(&self) -> SEQNEARENDIRQ_R {
        SEQNEARENDIRQ_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Sequence End IRQ"]
    #[inline(always)]
    pub fn seqatendirq(&self) -> SEQATENDIRQ_R {
        SEQATENDIRQ_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Beeper Sequence Repeat Value"]
    #[inline(always)]
    pub fn seqrepeat(&mut self) -> SEQREPEAT_W {
        SEQREPEAT_W { w: self }
    }
    #[doc = "Bit 8 - Beeper Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 10 - Tone A Start IRQ"]
    #[inline(always)]
    pub fn astartirq(&mut self) -> ASTARTIRQ_W {
        ASTARTIRQ_W { w: self }
    }
    #[doc = "Bit 11 - Tone A End IRQ"]
    #[inline(always)]
    pub fn aendirq(&mut self) -> AENDIRQ_W {
        AENDIRQ_W { w: self }
    }
    #[doc = "Bit 12 - Tone B Start IRQ"]
    #[inline(always)]
    pub fn bstartirq(&mut self) -> BSTARTIRQ_W {
        BSTARTIRQ_W { w: self }
    }
    #[doc = "Bit 13 - Tone B End IRQ"]
    #[inline(always)]
    pub fn bendirq(&mut self) -> BENDIRQ_W {
        BENDIRQ_W { w: self }
    }
    #[doc = "Bit 14 - Sequence 1 Cycle from End IRQ"]
    #[inline(always)]
    pub fn seqnearendirq(&mut self) -> SEQNEARENDIRQ_W {
        SEQNEARENDIRQ_W { w: self }
    }
    #[doc = "Bit 15 - Sequence End IRQ"]
    #[inline(always)]
    pub fn seqatendirq(&mut self) -> SEQATENDIRQ_W {
        SEQATENDIRQ_W { w: self }
    }
}
