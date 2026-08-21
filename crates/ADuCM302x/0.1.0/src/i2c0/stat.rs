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
#[doc = "Reader of field `STXF`"]
pub type STXF_R = crate::R<u8, u8>;
#[doc = "Reader of field `SRXF`"]
pub type SRXF_R = crate::R<u8, u8>;
#[doc = "Reader of field `MTXF`"]
pub type MTXF_R = crate::R<u8, u8>;
#[doc = "Reader of field `MRXF`"]
pub type MRXF_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SFLUSH`"]
pub struct SFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> SFLUSH_W<'a> {
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
#[doc = "Write proxy for field `MFLUSH`"]
pub struct MFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> MFLUSH_W<'a> {
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
impl R {
    #[doc = "Bits 0:1 - Slave Transmit FIFO Status"]
    #[inline(always)]
    pub fn stxf(&self) -> STXF_R {
        STXF_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 2:3 - Slave Receive FIFO Status"]
    #[inline(always)]
    pub fn srxf(&self) -> SRXF_R {
        SRXF_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bits 4:5 - Master Transmit FIFO Status"]
    #[inline(always)]
    pub fn mtxf(&self) -> MTXF_R {
        MTXF_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 6:7 - Master Receive FIFO Status"]
    #[inline(always)]
    pub fn mrxf(&self) -> MRXF_R {
        MRXF_R::new(((self.bits >> 6) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bit 8 - Flush the Slave Transmit FIFO"]
    #[inline(always)]
    pub fn sflush(&mut self) -> SFLUSH_W {
        SFLUSH_W { w: self }
    }
    #[doc = "Bit 9 - Flush the Master Transmit FIFO"]
    #[inline(always)]
    pub fn mflush(&mut self) -> MFLUSH_W {
        MFLUSH_W { w: self }
    }
}
