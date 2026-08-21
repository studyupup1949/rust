#[doc = "Reader of register CR0"]
pub type R = crate::R<u16, super::CR0>;
#[doc = "Writer for register CR0"]
pub type W = crate::W<u16, super::CR0>;
#[doc = "Register CR0 `reset()`'s with value 0x03c4"]
impl crate::ResetValue for super::CR0 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x03c4
    }
}
#[doc = "Reader of field `CNTEN`"]
pub type CNTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNTEN`"]
pub struct CNTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CNTEN_W<'a> {
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
#[doc = "Reader of field `ALMEN`"]
pub type ALMEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALMEN`"]
pub struct ALMEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ALMEN_W<'a> {
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
#[doc = "Reader of field `ALMINTEN`"]
pub type ALMINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALMINTEN`"]
pub struct ALMINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ALMINTEN_W<'a> {
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
#[doc = "Reader of field `TRMEN`"]
pub type TRMEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TRMEN`"]
pub struct TRMEN_W<'a> {
    w: &'a mut W,
}
impl<'a> TRMEN_W<'a> {
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
#[doc = "Reader of field `MOD60ALMEN`"]
pub type MOD60ALMEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MOD60ALMEN`"]
pub struct MOD60ALMEN_W<'a> {
    w: &'a mut W,
}
impl<'a> MOD60ALMEN_W<'a> {
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
#[doc = "Reader of field `MOD60ALM`"]
pub type MOD60ALM_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `MOD60ALM`"]
pub struct MOD60ALM_W<'a> {
    w: &'a mut W,
}
impl<'a> MOD60ALM_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x3f << 5)) | (((value as u16) & 0x3f) << 5);
        self.w
    }
}
#[doc = "Reader of field `MOD60ALMINTEN`"]
pub type MOD60ALMINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MOD60ALMINTEN`"]
pub struct MOD60ALMINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> MOD60ALMINTEN_W<'a> {
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
#[doc = "Reader of field `ISOINTEN`"]
pub type ISOINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ISOINTEN`"]
pub struct ISOINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ISOINTEN_W<'a> {
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
#[doc = "Reader of field `WPNDERRINTEN`"]
pub type WPNDERRINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WPNDERRINTEN`"]
pub struct WPNDERRINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> WPNDERRINTEN_W<'a> {
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
#[doc = "Reader of field `WSYNCINTEN`"]
pub type WSYNCINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WSYNCINTEN`"]
pub struct WSYNCINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> WSYNCINTEN_W<'a> {
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
#[doc = "Reader of field `WPNDINTEN`"]
pub type WPNDINTEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WPNDINTEN`"]
pub struct WPNDINTEN_W<'a> {
    w: &'a mut W,
}
impl<'a> WPNDINTEN_W<'a> {
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
    #[doc = "Bit 0 - Global Enable for the RTC"]
    #[inline(always)]
    pub fn cnten(&self) -> CNTEN_R {
        CNTEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable the RTC Alarm (Absolute) Operation"]
    #[inline(always)]
    pub fn almen(&self) -> ALMEN_R {
        ALMEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable ALMINT Sourced Alarm Interrupts to the CPU"]
    #[inline(always)]
    pub fn alminten(&self) -> ALMINTEN_R {
        ALMINTEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Enable RTC Digital Trimming"]
    #[inline(always)]
    pub fn trmen(&self) -> TRMEN_R {
        TRMEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Enable RTC Modulo-60 Counting of Time Past a Modulo-60 Boundary"]
    #[inline(always)]
    pub fn mod60almen(&self) -> MOD60ALMEN_R {
        MOD60ALMEN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 5:10 - Periodic, Modulo-60 Alarm Time in Prescaled RTC Time Units Beyond a Modulo-60 Boundary"]
    #[inline(always)]
    pub fn mod60alm(&self) -> MOD60ALM_R {
        MOD60ALM_R::new(((self.bits >> 5) & 0x3f) as u8)
    }
    #[doc = "Bit 11 - Enable Periodic Modulo-60 RTC Alarm Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn mod60alminten(&self) -> MOD60ALMINTEN_R {
        MOD60ALMINTEN_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Enable ISOINT Sourced Interrupts to the CPU When Isolation of the RTC Power Domain is Activated and Subsequently De-activated"]
    #[inline(always)]
    pub fn isointen(&self) -> ISOINTEN_R {
        ISOINTEN_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Enable Write Pending Error Sourced Interrupts to the CPU When an RTC Register-write Pending Error Occurs"]
    #[inline(always)]
    pub fn wpnderrinten(&self) -> WPNDERRINTEN_R {
        WPNDERRINTEN_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Enable Write Synchronization Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn wsyncinten(&self) -> WSYNCINTEN_R {
        WSYNCINTEN_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Enable Write Pending Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn wpndinten(&self) -> WPNDINTEN_R {
        WPNDINTEN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Global Enable for the RTC"]
    #[inline(always)]
    pub fn cnten(&mut self) -> CNTEN_W {
        CNTEN_W { w: self }
    }
    #[doc = "Bit 1 - Enable the RTC Alarm (Absolute) Operation"]
    #[inline(always)]
    pub fn almen(&mut self) -> ALMEN_W {
        ALMEN_W { w: self }
    }
    #[doc = "Bit 2 - Enable ALMINT Sourced Alarm Interrupts to the CPU"]
    #[inline(always)]
    pub fn alminten(&mut self) -> ALMINTEN_W {
        ALMINTEN_W { w: self }
    }
    #[doc = "Bit 3 - Enable RTC Digital Trimming"]
    #[inline(always)]
    pub fn trmen(&mut self) -> TRMEN_W {
        TRMEN_W { w: self }
    }
    #[doc = "Bit 4 - Enable RTC Modulo-60 Counting of Time Past a Modulo-60 Boundary"]
    #[inline(always)]
    pub fn mod60almen(&mut self) -> MOD60ALMEN_W {
        MOD60ALMEN_W { w: self }
    }
    #[doc = "Bits 5:10 - Periodic, Modulo-60 Alarm Time in Prescaled RTC Time Units Beyond a Modulo-60 Boundary"]
    #[inline(always)]
    pub fn mod60alm(&mut self) -> MOD60ALM_W {
        MOD60ALM_W { w: self }
    }
    #[doc = "Bit 11 - Enable Periodic Modulo-60 RTC Alarm Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn mod60alminten(&mut self) -> MOD60ALMINTEN_W {
        MOD60ALMINTEN_W { w: self }
    }
    #[doc = "Bit 12 - Enable ISOINT Sourced Interrupts to the CPU When Isolation of the RTC Power Domain is Activated and Subsequently De-activated"]
    #[inline(always)]
    pub fn isointen(&mut self) -> ISOINTEN_W {
        ISOINTEN_W { w: self }
    }
    #[doc = "Bit 13 - Enable Write Pending Error Sourced Interrupts to the CPU When an RTC Register-write Pending Error Occurs"]
    #[inline(always)]
    pub fn wpnderrinten(&mut self) -> WPNDERRINTEN_W {
        WPNDERRINTEN_W { w: self }
    }
    #[doc = "Bit 14 - Enable Write Synchronization Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn wsyncinten(&mut self) -> WSYNCINTEN_W {
        WSYNCINTEN_W { w: self }
    }
    #[doc = "Bit 15 - Enable Write Pending Sourced Interrupts to the CPU"]
    #[inline(always)]
    pub fn wpndinten(&mut self) -> WPNDINTEN_W {
        WPNDINTEN_W { w: self }
    }
}
