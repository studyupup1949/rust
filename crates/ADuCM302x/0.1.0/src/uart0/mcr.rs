#[doc = "Reader of register MCR"]
pub type R = crate::R<u16, super::MCR>;
#[doc = "Writer for register MCR"]
pub type W = crate::W<u16, super::MCR>;
#[doc = "Register MCR `reset()`'s with value 0"]
impl crate::ResetValue for super::MCR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `DTR`"]
pub type DTR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DTR`"]
pub struct DTR_W<'a> {
    w: &'a mut W,
}
impl<'a> DTR_W<'a> {
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
#[doc = "Reader of field `RTS`"]
pub type RTS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RTS`"]
pub struct RTS_W<'a> {
    w: &'a mut W,
}
impl<'a> RTS_W<'a> {
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
#[doc = "Reader of field `OUT1`"]
pub type OUT1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OUT1`"]
pub struct OUT1_W<'a> {
    w: &'a mut W,
}
impl<'a> OUT1_W<'a> {
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
#[doc = "Reader of field `OUT2`"]
pub type OUT2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OUT2`"]
pub struct OUT2_W<'a> {
    w: &'a mut W,
}
impl<'a> OUT2_W<'a> {
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
#[doc = "Reader of field `LOOPBACK`"]
pub type LOOPBACK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LOOPBACK`"]
pub struct LOOPBACK_W<'a> {
    w: &'a mut W,
}
impl<'a> LOOPBACK_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Data Terminal Ready"]
    #[inline(always)]
    pub fn dtr(&self) -> DTR_R {
        DTR_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Request to Send"]
    #[inline(always)]
    pub fn rts(&self) -> RTS_R {
        RTS_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Output 1"]
    #[inline(always)]
    pub fn out1(&self) -> OUT1_R {
        OUT1_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Output 2"]
    #[inline(always)]
    pub fn out2(&self) -> OUT2_R {
        OUT2_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Loopback Mode"]
    #[inline(always)]
    pub fn loopback(&self) -> LOOPBACK_R {
        LOOPBACK_R::new(((self.bits >> 4) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Data Terminal Ready"]
    #[inline(always)]
    pub fn dtr(&mut self) -> DTR_W {
        DTR_W { w: self }
    }
    #[doc = "Bit 1 - Request to Send"]
    #[inline(always)]
    pub fn rts(&mut self) -> RTS_W {
        RTS_W { w: self }
    }
    #[doc = "Bit 2 - Output 1"]
    #[inline(always)]
    pub fn out1(&mut self) -> OUT1_W {
        OUT1_W { w: self }
    }
    #[doc = "Bit 3 - Output 2"]
    #[inline(always)]
    pub fn out2(&mut self) -> OUT2_W {
        OUT2_W { w: self }
    }
    #[doc = "Bit 4 - Loopback Mode"]
    #[inline(always)]
    pub fn loopback(&mut self) -> LOOPBACK_W {
        LOOPBACK_W { w: self }
    }
}
