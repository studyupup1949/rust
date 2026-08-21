#[doc = "Reader of register MSTAT"]
pub type R = crate::R<u16, super::MSTAT>;
#[doc = "Writer for register MSTAT"]
pub type W = crate::W<u16, super::MSTAT>;
#[doc = "Register MSTAT `reset()`'s with value 0x6000"]
impl crate::ResetValue for super::MSTAT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x6000
    }
}
#[doc = "Master Transmit FIFO Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum MTXF_A {
    #[doc = "0: FIFO Empty."]
    FIFO_EMPTY = 0,
    #[doc = "2: 1 byte in FIFO."]
    FIFO_1BYTE = 2,
    #[doc = "3: FIFO Full."]
    FIFO_FULL = 3,
}
impl From<MTXF_A> for u8 {
    #[inline(always)]
    fn from(variant: MTXF_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `MTXF`"]
pub type MTXF_R = crate::R<u8, MTXF_A>;
impl MTXF_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, MTXF_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(MTXF_A::FIFO_EMPTY),
            2 => Val(MTXF_A::FIFO_1BYTE),
            3 => Val(MTXF_A::FIFO_FULL),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `FIFO_EMPTY`"]
    #[inline(always)]
    pub fn is_fifo_empty(&self) -> bool {
        *self == MTXF_A::FIFO_EMPTY
    }
    #[doc = "Checks if the value of the field is `FIFO_1BYTE`"]
    #[inline(always)]
    pub fn is_fifo_1byte(&self) -> bool {
        *self == MTXF_A::FIFO_1BYTE
    }
    #[doc = "Checks if the value of the field is `FIFO_FULL`"]
    #[inline(always)]
    pub fn is_fifo_full(&self) -> bool {
        *self == MTXF_A::FIFO_FULL
    }
}
#[doc = "Reader of field `MTXREQ`"]
pub type MTXREQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MTXREQ`"]
pub struct MTXREQ_W<'a> {
    w: &'a mut W,
}
impl<'a> MTXREQ_W<'a> {
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
#[doc = "Reader of field `MRXREQ`"]
pub type MRXREQ_R = crate::R<bool, bool>;
#[doc = "Reader of field `NACKADDR`"]
pub type NACKADDR_R = crate::R<bool, bool>;
#[doc = "Reader of field `ALOST`"]
pub type ALOST_R = crate::R<bool, bool>;
#[doc = "Reader of field `MBUSY`"]
pub type MBUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `NACKDATA`"]
pub type NACKDATA_R = crate::R<bool, bool>;
#[doc = "Reader of field `TCOMP`"]
pub type TCOMP_R = crate::R<bool, bool>;
#[doc = "Reader of field `MRXOVR`"]
pub type MRXOVR_R = crate::R<bool, bool>;
#[doc = "Reader of field `LINEBUSY`"]
pub type LINEBUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `MSTOP`"]
pub type MSTOP_R = crate::R<bool, bool>;
#[doc = "Reader of field `MTXUNDR`"]
pub type MTXUNDR_R = crate::R<bool, bool>;
#[doc = "Reader of field `SDAFILT`"]
pub type SDAFILT_R = crate::R<bool, bool>;
#[doc = "Reader of field `SCLFILT`"]
pub type SCLFILT_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bits 0:1 - Master Transmit FIFO Status"]
    #[inline(always)]
    pub fn mtxf(&self) -> MTXF_R {
        MTXF_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bit 2 - Master Transmit Request/Clear Master Transmit Interrupt"]
    #[inline(always)]
    pub fn mtxreq(&self) -> MTXREQ_R {
        MTXREQ_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Master Receive Request"]
    #[inline(always)]
    pub fn mrxreq(&self) -> MRXREQ_R {
        MRXREQ_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - ACK Not Received in Response to an Address"]
    #[inline(always)]
    pub fn nackaddr(&self) -> NACKADDR_R {
        NACKADDR_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Arbitration Lost"]
    #[inline(always)]
    pub fn alost(&self) -> ALOST_R {
        ALOST_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Master Busy"]
    #[inline(always)]
    pub fn mbusy(&self) -> MBUSY_R {
        MBUSY_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - ACK Not Received in Response to Data Write"]
    #[inline(always)]
    pub fn nackdata(&self) -> NACKDATA_R {
        NACKDATA_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Transaction Complete or Stop Detected"]
    #[inline(always)]
    pub fn tcomp(&self) -> TCOMP_R {
        TCOMP_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Master Receive FIFO Overflow"]
    #[inline(always)]
    pub fn mrxovr(&self) -> MRXOVR_R {
        MRXOVR_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Line is Busy"]
    #[inline(always)]
    pub fn linebusy(&self) -> LINEBUSY_R {
        LINEBUSY_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - STOP Driven by This I2C Master"]
    #[inline(always)]
    pub fn mstop(&self) -> MSTOP_R {
        MSTOP_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Master Transmit Underflow"]
    #[inline(always)]
    pub fn mtxundr(&self) -> MTXUNDR_R {
        MTXUNDR_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - State of SDA Line"]
    #[inline(always)]
    pub fn sdafilt(&self) -> SDAFILT_R {
        SDAFILT_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - State of SCL Line"]
    #[inline(always)]
    pub fn sclfilt(&self) -> SCLFILT_R {
        SCLFILT_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 2 - Master Transmit Request/Clear Master Transmit Interrupt"]
    #[inline(always)]
    pub fn mtxreq(&mut self) -> MTXREQ_W {
        MTXREQ_W { w: self }
    }
}
