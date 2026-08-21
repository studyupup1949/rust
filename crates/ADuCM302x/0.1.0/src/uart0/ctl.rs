#[doc = "Reader of register CTL"]
pub type R = crate::R<u16, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u16, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0x0100"]
impl crate::ResetValue for super::CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0100
    }
}
#[doc = "Reader of field `FORCECLK`"]
pub type FORCECLK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FORCECLK`"]
pub struct FORCECLK_W<'a> {
    w: &'a mut W,
}
impl<'a> FORCECLK_W<'a> {
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
#[doc = "Invert Receiver Line\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RXINV_A {
    #[doc = "0: Don't invert receiver line (idling high)."]
    NOTINV_RX = 0,
    #[doc = "1: Invert receiver line (idling low)."]
    INV_RX = 1,
}
impl From<RXINV_A> for bool {
    #[inline(always)]
    fn from(variant: RXINV_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `RXINV`"]
pub type RXINV_R = crate::R<bool, RXINV_A>;
impl RXINV_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> RXINV_A {
        match self.bits {
            false => RXINV_A::NOTINV_RX,
            true => RXINV_A::INV_RX,
        }
    }
    #[doc = "Checks if the value of the field is `NOTINV_RX`"]
    #[inline(always)]
    pub fn is_notinv_rx(&self) -> bool {
        *self == RXINV_A::NOTINV_RX
    }
    #[doc = "Checks if the value of the field is `INV_RX`"]
    #[inline(always)]
    pub fn is_inv_rx(&self) -> bool {
        *self == RXINV_A::INV_RX
    }
}
#[doc = "Write proxy for field `RXINV`"]
pub struct RXINV_W<'a> {
    w: &'a mut W,
}
impl<'a> RXINV_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: RXINV_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Don't invert receiver line (idling high)."]
    #[inline(always)]
    pub fn notinv_rx(self) -> &'a mut W {
        self.variant(RXINV_A::NOTINV_RX)
    }
    #[doc = "Invert receiver line (idling low)."]
    #[inline(always)]
    pub fn inv_rx(self) -> &'a mut W {
        self.variant(RXINV_A::INV_RX)
    }
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
#[doc = "Reader of field `REV`"]
pub type REV_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 1 - Force UCLK on"]
    #[inline(always)]
    pub fn forceclk(&self) -> FORCECLK_R {
        FORCECLK_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Invert Receiver Line"]
    #[inline(always)]
    pub fn rxinv(&self) -> RXINV_R {
        RXINV_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 8:15 - UART Revision ID"]
    #[inline(always)]
    pub fn rev(&self) -> REV_R {
        REV_R::new(((self.bits >> 8) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bit 1 - Force UCLK on"]
    #[inline(always)]
    pub fn forceclk(&mut self) -> FORCECLK_W {
        FORCECLK_W { w: self }
    }
    #[doc = "Bit 4 - Invert Receiver Line"]
    #[inline(always)]
    pub fn rxinv(&mut self) -> RXINV_W {
        RXINV_W { w: self }
    }
}
