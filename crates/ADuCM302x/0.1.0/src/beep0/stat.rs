#[doc = "Reader of register STAT"]
pub type R = crate::R<u16, super::STAT>;
#[doc = "Writer for register STAT"]
pub type W = crate::W<u16, super::STAT>;
#[doc = "Register STAT `reset()`'s with value 0"]
impl crate::ResetValue for super::STAT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SEQREMAIN`"]
pub type SEQREMAIN_R = crate::R<u8, u8>;
#[doc = "Reader of field `BUSY`"]
pub type BUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `ASTARTED`"]
pub type ASTARTED_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ASTARTED`"]
pub struct ASTARTED_W<'a> {
    w: &'a mut W,
}
impl<'a> ASTARTED_W<'a> {
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
#[doc = "Reader of field `AENDED`"]
pub type AENDED_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AENDED`"]
pub struct AENDED_W<'a> {
    w: &'a mut W,
}
impl<'a> AENDED_W<'a> {
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
#[doc = "Reader of field `BSTARTED`"]
pub type BSTARTED_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BSTARTED`"]
pub struct BSTARTED_W<'a> {
    w: &'a mut W,
}
impl<'a> BSTARTED_W<'a> {
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
#[doc = "Reader of field `BENDED`"]
pub type BENDED_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BENDED`"]
pub struct BENDED_W<'a> {
    w: &'a mut W,
}
impl<'a> BENDED_W<'a> {
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
#[doc = "Reader of field `SEQNEAREND`"]
pub type SEQNEAREND_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SEQNEAREND`"]
pub struct SEQNEAREND_W<'a> {
    w: &'a mut W,
}
impl<'a> SEQNEAREND_W<'a> {
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
#[doc = "Reader of field `SEQENDED`"]
pub type SEQENDED_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SEQENDED`"]
pub struct SEQENDED_W<'a> {
    w: &'a mut W,
}
impl<'a> SEQENDED_W<'a> {
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
    #[doc = "Bits 0:7 - Remaining Tone-pair Iterations to Play in Sequence Mode"]
    #[inline(always)]
    pub fn seqremain(&self) -> SEQREMAIN_R {
        SEQREMAIN_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 8 - Beeper is Busy"]
    #[inline(always)]
    pub fn busy(&self) -> BUSY_R {
        BUSY_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Tone A Has Started"]
    #[inline(always)]
    pub fn astarted(&self) -> ASTARTED_R {
        ASTARTED_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Tone A Has Ended"]
    #[inline(always)]
    pub fn aended(&self) -> AENDED_R {
        AENDED_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Tone B Has Started"]
    #[inline(always)]
    pub fn bstarted(&self) -> BSTARTED_R {
        BSTARTED_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Tone B Has Ended"]
    #[inline(always)]
    pub fn bended(&self) -> BENDED_R {
        BENDED_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Sequencer Last Tone-pair Has Started"]
    #[inline(always)]
    pub fn seqnearend(&self) -> SEQNEAREND_R {
        SEQNEAREND_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Sequencer Has Ended"]
    #[inline(always)]
    pub fn seqended(&self) -> SEQENDED_R {
        SEQENDED_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 10 - Tone A Has Started"]
    #[inline(always)]
    pub fn astarted(&mut self) -> ASTARTED_W {
        ASTARTED_W { w: self }
    }
    #[doc = "Bit 11 - Tone A Has Ended"]
    #[inline(always)]
    pub fn aended(&mut self) -> AENDED_W {
        AENDED_W { w: self }
    }
    #[doc = "Bit 12 - Tone B Has Started"]
    #[inline(always)]
    pub fn bstarted(&mut self) -> BSTARTED_W {
        BSTARTED_W { w: self }
    }
    #[doc = "Bit 13 - Tone B Has Ended"]
    #[inline(always)]
    pub fn bended(&mut self) -> BENDED_W {
        BENDED_W { w: self }
    }
    #[doc = "Bit 14 - Sequencer Last Tone-pair Has Started"]
    #[inline(always)]
    pub fn seqnearend(&mut self) -> SEQNEAREND_W {
        SEQNEAREND_W { w: self }
    }
    #[doc = "Bit 15 - Sequencer Has Ended"]
    #[inline(always)]
    pub fn seqended(&mut self) -> SEQENDED_W {
        SEQENDED_W { w: self }
    }
}
