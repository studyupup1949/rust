#[doc = "Reader of register CTL"]
pub type R = crate::R<u16, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u16, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0"]
impl crate::ResetValue for super::CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SPIEN`"]
pub type SPIEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPIEN`"]
pub struct SPIEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SPIEN_W<'a> {
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
#[doc = "Reader of field `MASEN`"]
pub type MASEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MASEN`"]
pub struct MASEN_W<'a> {
    w: &'a mut W,
}
impl<'a> MASEN_W<'a> {
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
#[doc = "Reader of field `CPHA`"]
pub type CPHA_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CPHA`"]
pub struct CPHA_W<'a> {
    w: &'a mut W,
}
impl<'a> CPHA_W<'a> {
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
#[doc = "Reader of field `CPOL`"]
pub type CPOL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CPOL`"]
pub struct CPOL_W<'a> {
    w: &'a mut W,
}
impl<'a> CPOL_W<'a> {
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
#[doc = "Reader of field `WOM`"]
pub type WOM_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WOM`"]
pub struct WOM_W<'a> {
    w: &'a mut W,
}
impl<'a> WOM_W<'a> {
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
#[doc = "Reader of field `LSB`"]
pub type LSB_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LSB`"]
pub struct LSB_W<'a> {
    w: &'a mut W,
}
impl<'a> LSB_W<'a> {
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
#[doc = "Reader of field `TIM`"]
pub type TIM_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TIM`"]
pub struct TIM_W<'a> {
    w: &'a mut W,
}
impl<'a> TIM_W<'a> {
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
#[doc = "Reader of field `ZEN`"]
pub type ZEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ZEN`"]
pub struct ZEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ZEN_W<'a> {
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
#[doc = "Reader of field `RXOF`"]
pub type RXOF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RXOF`"]
pub struct RXOF_W<'a> {
    w: &'a mut W,
}
impl<'a> RXOF_W<'a> {
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
#[doc = "Reader of field `OEN`"]
pub type OEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OEN`"]
pub struct OEN_W<'a> {
    w: &'a mut W,
}
impl<'a> OEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u16) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `CON`"]
pub type CON_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CON`"]
pub struct CON_W<'a> {
    w: &'a mut W,
}
impl<'a> CON_W<'a> {
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
#[doc = "Reader of field `RFLUSH`"]
pub type RFLUSH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RFLUSH`"]
pub struct RFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> RFLUSH_W<'a> {
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
#[doc = "Reader of field `TFLUSH`"]
pub type TFLUSH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TFLUSH`"]
pub struct TFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> TFLUSH_W<'a> {
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
#[doc = "Reader of field `CSRST`"]
pub type CSRST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CSRST`"]
pub struct CSRST_W<'a> {
    w: &'a mut W,
}
impl<'a> CSRST_W<'a> {
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
impl R {
    #[doc = "Bit 0 - SPI Enable"]
    #[inline(always)]
    pub fn spien(&self) -> SPIEN_R {
        SPIEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Master Mode Enable"]
    #[inline(always)]
    pub fn masen(&self) -> MASEN_R {
        MASEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Serial Clock Phase Mode"]
    #[inline(always)]
    pub fn cpha(&self) -> CPHA_R {
        CPHA_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Serial Clock Polarity"]
    #[inline(always)]
    pub fn cpol(&self) -> CPOL_R {
        CPOL_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - SPI Wired-OR Mode"]
    #[inline(always)]
    pub fn wom(&self) -> WOM_R {
        WOM_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - LSB First Transfer Enable"]
    #[inline(always)]
    pub fn lsb(&self) -> LSB_R {
        LSB_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - SPI Transfer and Interrupt Mode"]
    #[inline(always)]
    pub fn tim(&self) -> TIM_R {
        TIM_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Transmit Zeros Enable"]
    #[inline(always)]
    pub fn zen(&self) -> ZEN_R {
        ZEN_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Rx Overflow Overwrite Enable"]
    #[inline(always)]
    pub fn rxof(&self) -> RXOF_R {
        RXOF_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Slave MISO Output Enable"]
    #[inline(always)]
    pub fn oen(&self) -> OEN_R {
        OEN_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Loopback Enable"]
    #[inline(always)]
    pub fn loopback(&self) -> LOOPBACK_R {
        LOOPBACK_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Continuous Transfer Enable"]
    #[inline(always)]
    pub fn con(&self) -> CON_R {
        CON_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - SPI Rx FIFO Flush Enable"]
    #[inline(always)]
    pub fn rflush(&self) -> RFLUSH_R {
        RFLUSH_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - SPI Tx FIFO Flush Enable"]
    #[inline(always)]
    pub fn tflush(&self) -> TFLUSH_R {
        TFLUSH_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Reset Mode for CS Error Bit"]
    #[inline(always)]
    pub fn csrst(&self) -> CSRST_R {
        CSRST_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SPI Enable"]
    #[inline(always)]
    pub fn spien(&mut self) -> SPIEN_W {
        SPIEN_W { w: self }
    }
    #[doc = "Bit 1 - Master Mode Enable"]
    #[inline(always)]
    pub fn masen(&mut self) -> MASEN_W {
        MASEN_W { w: self }
    }
    #[doc = "Bit 2 - Serial Clock Phase Mode"]
    #[inline(always)]
    pub fn cpha(&mut self) -> CPHA_W {
        CPHA_W { w: self }
    }
    #[doc = "Bit 3 - Serial Clock Polarity"]
    #[inline(always)]
    pub fn cpol(&mut self) -> CPOL_W {
        CPOL_W { w: self }
    }
    #[doc = "Bit 4 - SPI Wired-OR Mode"]
    #[inline(always)]
    pub fn wom(&mut self) -> WOM_W {
        WOM_W { w: self }
    }
    #[doc = "Bit 5 - LSB First Transfer Enable"]
    #[inline(always)]
    pub fn lsb(&mut self) -> LSB_W {
        LSB_W { w: self }
    }
    #[doc = "Bit 6 - SPI Transfer and Interrupt Mode"]
    #[inline(always)]
    pub fn tim(&mut self) -> TIM_W {
        TIM_W { w: self }
    }
    #[doc = "Bit 7 - Transmit Zeros Enable"]
    #[inline(always)]
    pub fn zen(&mut self) -> ZEN_W {
        ZEN_W { w: self }
    }
    #[doc = "Bit 8 - Rx Overflow Overwrite Enable"]
    #[inline(always)]
    pub fn rxof(&mut self) -> RXOF_W {
        RXOF_W { w: self }
    }
    #[doc = "Bit 9 - Slave MISO Output Enable"]
    #[inline(always)]
    pub fn oen(&mut self) -> OEN_W {
        OEN_W { w: self }
    }
    #[doc = "Bit 10 - Loopback Enable"]
    #[inline(always)]
    pub fn loopback(&mut self) -> LOOPBACK_W {
        LOOPBACK_W { w: self }
    }
    #[doc = "Bit 11 - Continuous Transfer Enable"]
    #[inline(always)]
    pub fn con(&mut self) -> CON_W {
        CON_W { w: self }
    }
    #[doc = "Bit 12 - SPI Rx FIFO Flush Enable"]
    #[inline(always)]
    pub fn rflush(&mut self) -> RFLUSH_W {
        RFLUSH_W { w: self }
    }
    #[doc = "Bit 13 - SPI Tx FIFO Flush Enable"]
    #[inline(always)]
    pub fn tflush(&mut self) -> TFLUSH_W {
        TFLUSH_W { w: self }
    }
    #[doc = "Bit 14 - Reset Mode for CS Error Bit"]
    #[inline(always)]
    pub fn csrst(&mut self) -> CSRST_W {
        CSRST_W { w: self }
    }
}
