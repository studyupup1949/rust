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
#[doc = "Reader of field `DONE0`"]
pub type DONE0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE0`"]
pub struct DONE0_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE0_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u16) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `DONE1`"]
pub type DONE1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE1`"]
pub struct DONE1_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE1_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u16) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `DONE2`"]
pub type DONE2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE2`"]
pub struct DONE2_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u16) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `DONE3`"]
pub type DONE3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE3`"]
pub struct DONE3_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE3_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u16) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `DONE4`"]
pub type DONE4_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE4`"]
pub struct DONE4_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE4_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `DONE5`"]
pub type DONE5_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE5`"]
pub struct DONE5_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE5_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u16) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `DONE6`"]
pub type DONE6_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE6`"]
pub struct DONE6_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE6_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u16) & 0x01) << 6);
        self.w
    }
}
#[doc = "Reader of field `DONE7`"]
pub type DONE7_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DONE7`"]
pub struct DONE7_W<'a> {
    w: &'a mut W,
}
impl<'a> DONE7_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u16) & 0x01) << 7);
        self.w
    }
}
#[doc = "Reader of field `BATDONE`"]
pub type BATDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BATDONE`"]
pub struct BATDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> BATDONE_W<'a> {
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
#[doc = "Reader of field `TMPDONE`"]
pub type TMPDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TMPDONE`"]
pub struct TMPDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> TMPDONE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u16) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `TMP2DONE`"]
pub type TMP2DONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TMP2DONE`"]
pub struct TMP2DONE_W<'a> {
    w: &'a mut W,
}
impl<'a> TMP2DONE_W<'a> {
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
#[doc = "Reader of field `CALDONE`"]
pub type CALDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CALDONE`"]
pub struct CALDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> CALDONE_W<'a> {
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
#[doc = "Reader of field `RDY`"]
pub type RDY_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RDY`"]
pub struct RDY_W<'a> {
    w: &'a mut W,
}
impl<'a> RDY_W<'a> {
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
    #[doc = "Bit 0 - Conversion Done on Channel 0"]
    #[inline(always)]
    pub fn done0(&self) -> DONE0_R {
        DONE0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Conversion Done on Channel 1"]
    #[inline(always)]
    pub fn done1(&self) -> DONE1_R {
        DONE1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Conversion Done on Channel 2"]
    #[inline(always)]
    pub fn done2(&self) -> DONE2_R {
        DONE2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Conversion Done on Channel 3"]
    #[inline(always)]
    pub fn done3(&self) -> DONE3_R {
        DONE3_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Conversion Done on Channel 4"]
    #[inline(always)]
    pub fn done4(&self) -> DONE4_R {
        DONE4_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Conversion Done on Channel 5"]
    #[inline(always)]
    pub fn done5(&self) -> DONE5_R {
        DONE5_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Conversion Done on Channel 6"]
    #[inline(always)]
    pub fn done6(&self) -> DONE6_R {
        DONE6_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Conversion Done on Channel 7"]
    #[inline(always)]
    pub fn done7(&self) -> DONE7_R {
        DONE7_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Conversion Done - Battery Monitoring"]
    #[inline(always)]
    pub fn batdone(&self) -> BATDONE_R {
        BATDONE_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Conversion Done for Temperature Sensing"]
    #[inline(always)]
    pub fn tmpdone(&self) -> TMPDONE_R {
        TMPDONE_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Conversion Done for Temperature Sensing 2"]
    #[inline(always)]
    pub fn tmp2done(&self) -> TMP2DONE_R {
        TMP2DONE_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Calibration Done"]
    #[inline(always)]
    pub fn caldone(&self) -> CALDONE_R {
        CALDONE_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - ADC Ready to Start Converting"]
    #[inline(always)]
    pub fn rdy(&self) -> RDY_R {
        RDY_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Conversion Done on Channel 0"]
    #[inline(always)]
    pub fn done0(&mut self) -> DONE0_W {
        DONE0_W { w: self }
    }
    #[doc = "Bit 1 - Conversion Done on Channel 1"]
    #[inline(always)]
    pub fn done1(&mut self) -> DONE1_W {
        DONE1_W { w: self }
    }
    #[doc = "Bit 2 - Conversion Done on Channel 2"]
    #[inline(always)]
    pub fn done2(&mut self) -> DONE2_W {
        DONE2_W { w: self }
    }
    #[doc = "Bit 3 - Conversion Done on Channel 3"]
    #[inline(always)]
    pub fn done3(&mut self) -> DONE3_W {
        DONE3_W { w: self }
    }
    #[doc = "Bit 4 - Conversion Done on Channel 4"]
    #[inline(always)]
    pub fn done4(&mut self) -> DONE4_W {
        DONE4_W { w: self }
    }
    #[doc = "Bit 5 - Conversion Done on Channel 5"]
    #[inline(always)]
    pub fn done5(&mut self) -> DONE5_W {
        DONE5_W { w: self }
    }
    #[doc = "Bit 6 - Conversion Done on Channel 6"]
    #[inline(always)]
    pub fn done6(&mut self) -> DONE6_W {
        DONE6_W { w: self }
    }
    #[doc = "Bit 7 - Conversion Done on Channel 7"]
    #[inline(always)]
    pub fn done7(&mut self) -> DONE7_W {
        DONE7_W { w: self }
    }
    #[doc = "Bit 8 - Conversion Done - Battery Monitoring"]
    #[inline(always)]
    pub fn batdone(&mut self) -> BATDONE_W {
        BATDONE_W { w: self }
    }
    #[doc = "Bit 9 - Conversion Done for Temperature Sensing"]
    #[inline(always)]
    pub fn tmpdone(&mut self) -> TMPDONE_W {
        TMPDONE_W { w: self }
    }
    #[doc = "Bit 10 - Conversion Done for Temperature Sensing 2"]
    #[inline(always)]
    pub fn tmp2done(&mut self) -> TMP2DONE_W {
        TMP2DONE_W { w: self }
    }
    #[doc = "Bit 14 - Calibration Done"]
    #[inline(always)]
    pub fn caldone(&mut self) -> CALDONE_W {
        CALDONE_W { w: self }
    }
    #[doc = "Bit 15 - ADC Ready to Start Converting"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RDY_W {
        RDY_W { w: self }
    }
}
