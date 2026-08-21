#[doc = "Reader of register CTL"]
pub type R = crate::R<u16, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u16, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0x0a"]
impl crate::ResetValue for super::CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0a
    }
}
#[doc = "Reader of field `PRE`"]
pub type PRE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PRE`"]
pub struct PRE_W<'a> {
    w: &'a mut W,
}
impl<'a> PRE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u16) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `UP`"]
pub type UP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `UP`"]
pub struct UP_W<'a> {
    w: &'a mut W,
}
impl<'a> UP_W<'a> {
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
#[doc = "Reader of field `MODE`"]
pub type MODE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MODE`"]
pub struct MODE_W<'a> {
    w: &'a mut W,
}
impl<'a> MODE_W<'a> {
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
#[doc = "Reader of field `EN`"]
pub type EN_R = crate::R<bool, bool>;
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `CLK`"]
pub type CLK_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CLK`"]
pub struct CLK_W<'a> {
    w: &'a mut W,
}
impl<'a> CLK_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 5)) | (((value as u16) & 0x03) << 5);
        self.w
    }
}
#[doc = "Reader of field `RLD`"]
pub type RLD_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RLD`"]
pub struct RLD_W<'a> {
    w: &'a mut W,
}
impl<'a> RLD_W<'a> {
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
#[doc = "Reader of field `EVTRANGE`"]
pub type EVTRANGE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `EVTRANGE`"]
pub struct EVTRANGE_W<'a> {
    w: &'a mut W,
}
impl<'a> EVTRANGE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x1f << 8)) | (((value as u16) & 0x1f) << 8);
        self.w
    }
}
#[doc = "Reader of field `EVTEN`"]
pub type EVTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EVTEN`"]
pub struct EVTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> EVTEN_W<'a> {
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
#[doc = "Reader of field `RSTEN`"]
pub type RSTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RSTEN`"]
pub struct RSTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> RSTEN_W<'a> {
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
#[doc = "Reader of field `SYNCBYP`"]
pub type SYNCBYP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SYNCBYP`"]
pub struct SYNCBYP_W<'a> {
    w: &'a mut W,
}
impl<'a> SYNCBYP_W<'a> {
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
    #[doc = "Bits 0:1 - Prescaler"]
    #[inline(always)]
    pub fn pre(&self) -> PRE_R {
        PRE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bit 2 - Count up"]
    #[inline(always)]
    pub fn up(&self) -> UP_R {
        UP_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Timer Mode"]
    #[inline(always)]
    pub fn mode(&self) -> MODE_R {
        MODE_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Timer Enable"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 5:6 - Clock Select"]
    #[inline(always)]
    pub fn clk(&self) -> CLK_R {
        CLK_R::new(((self.bits >> 5) & 0x03) as u8)
    }
    #[doc = "Bit 7 - Reload Control"]
    #[inline(always)]
    pub fn rld(&self) -> RLD_R {
        RLD_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bits 8:12 - Event Select Range"]
    #[inline(always)]
    pub fn evtrange(&self) -> EVTRANGE_R {
        EVTRANGE_R::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bit 13 - Event Select"]
    #[inline(always)]
    pub fn evten(&self) -> EVTEN_R {
        EVTEN_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Counter and Prescale Reset Enable"]
    #[inline(always)]
    pub fn rsten(&self) -> RSTEN_R {
        RSTEN_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Synchronization Bypass"]
    #[inline(always)]
    pub fn syncbyp(&self) -> SYNCBYP_R {
        SYNCBYP_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Prescaler"]
    #[inline(always)]
    pub fn pre(&mut self) -> PRE_W {
        PRE_W { w: self }
    }
    #[doc = "Bit 2 - Count up"]
    #[inline(always)]
    pub fn up(&mut self) -> UP_W {
        UP_W { w: self }
    }
    #[doc = "Bit 3 - Timer Mode"]
    #[inline(always)]
    pub fn mode(&mut self) -> MODE_W {
        MODE_W { w: self }
    }
    #[doc = "Bit 4 - Timer Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bits 5:6 - Clock Select"]
    #[inline(always)]
    pub fn clk(&mut self) -> CLK_W {
        CLK_W { w: self }
    }
    #[doc = "Bit 7 - Reload Control"]
    #[inline(always)]
    pub fn rld(&mut self) -> RLD_W {
        RLD_W { w: self }
    }
    #[doc = "Bits 8:12 - Event Select Range"]
    #[inline(always)]
    pub fn evtrange(&mut self) -> EVTRANGE_W {
        EVTRANGE_W { w: self }
    }
    #[doc = "Bit 13 - Event Select"]
    #[inline(always)]
    pub fn evten(&mut self) -> EVTEN_W {
        EVTEN_W { w: self }
    }
    #[doc = "Bit 14 - Counter and Prescale Reset Enable"]
    #[inline(always)]
    pub fn rsten(&mut self) -> RSTEN_W {
        RSTEN_W { w: self }
    }
    #[doc = "Bit 15 - Synchronization Bypass"]
    #[inline(always)]
    pub fn syncbyp(&mut self) -> SYNCBYP_W {
        SYNCBYP_W { w: self }
    }
}
