#[doc = "Reader of register IEN"]
pub type R = crate::R<u32, super::IEN>;
#[doc = "Writer for register IEN"]
pub type W = crate::W<u32, super::IEN>;
#[doc = "Register IEN `reset()`'s with value 0x60"]
impl crate::ResetValue for super::IEN {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x60
    }
}
#[doc = "Reader of field `CMDCMPLT`"]
pub type CMDCMPLT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CMDCMPLT`"]
pub struct CMDCMPLT_W<'a> {
    w: &'a mut W,
}
impl<'a> CMDCMPLT_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `WRALCMPLT`"]
pub type WRALCMPLT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `WRALCMPLT`"]
pub struct WRALCMPLT_W<'a> {
    w: &'a mut W,
}
impl<'a> WRALCMPLT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `CMDFAIL`"]
pub type CMDFAIL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CMDFAIL`"]
pub struct CMDFAIL_W<'a> {
    w: &'a mut W,
}
impl<'a> CMDFAIL_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Control 2-bit ECC Error Events\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ECC_ERROR_A {
    #[doc = "0: Do not generate a response to ECC events"]
    NONE_ERR = 0,
    #[doc = "1: Generate Bus Errors in response to ECC events"]
    BUS_ERR_ERR = 1,
    #[doc = "2: Generate IRQs in response to ECC events"]
    IRQ_ERR = 2,
}
impl From<ECC_ERROR_A> for u8 {
    #[inline(always)]
    fn from(variant: ECC_ERROR_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `ECC_ERROR`"]
pub type ECC_ERROR_R = crate::R<u8, ECC_ERROR_A>;
impl ECC_ERROR_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, ECC_ERROR_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(ECC_ERROR_A::NONE_ERR),
            1 => Val(ECC_ERROR_A::BUS_ERR_ERR),
            2 => Val(ECC_ERROR_A::IRQ_ERR),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `NONE_ERR`"]
    #[inline(always)]
    pub fn is_none_err(&self) -> bool {
        *self == ECC_ERROR_A::NONE_ERR
    }
    #[doc = "Checks if the value of the field is `BUS_ERR_ERR`"]
    #[inline(always)]
    pub fn is_bus_err_err(&self) -> bool {
        *self == ECC_ERROR_A::BUS_ERR_ERR
    }
    #[doc = "Checks if the value of the field is `IRQ_ERR`"]
    #[inline(always)]
    pub fn is_irq_err(&self) -> bool {
        *self == ECC_ERROR_A::IRQ_ERR
    }
}
#[doc = "Write proxy for field `ECC_ERROR`"]
pub struct ECC_ERROR_W<'a> {
    w: &'a mut W,
}
impl<'a> ECC_ERROR_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: ECC_ERROR_A) -> &'a mut W {
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "Do not generate a response to ECC events"]
    #[inline(always)]
    pub fn none_err(self) -> &'a mut W {
        self.variant(ECC_ERROR_A::NONE_ERR)
    }
    #[doc = "Generate Bus Errors in response to ECC events"]
    #[inline(always)]
    pub fn bus_err_err(self) -> &'a mut W {
        self.variant(ECC_ERROR_A::BUS_ERR_ERR)
    }
    #[doc = "Generate IRQs in response to ECC events"]
    #[inline(always)]
    pub fn irq_err(self) -> &'a mut W {
        self.variant(ECC_ERROR_A::IRQ_ERR)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 6)) | (((value as u32) & 0x03) << 6);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Command Complete Interrupt Enable"]
    #[inline(always)]
    pub fn cmdcmplt(&self) -> CMDCMPLT_R {
        CMDCMPLT_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Write Almost Complete Interrupt Enable"]
    #[inline(always)]
    pub fn wralcmplt(&self) -> WRALCMPLT_R {
        WRALCMPLT_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Command Fail Interrupt Enable"]
    #[inline(always)]
    pub fn cmdfail(&self) -> CMDFAIL_R {
        CMDFAIL_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bits 6:7 - Control 2-bit ECC Error Events"]
    #[inline(always)]
    pub fn ecc_error(&self) -> ECC_ERROR_R {
        ECC_ERROR_R::new(((self.bits >> 6) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Command Complete Interrupt Enable"]
    #[inline(always)]
    pub fn cmdcmplt(&mut self) -> CMDCMPLT_W {
        CMDCMPLT_W { w: self }
    }
    #[doc = "Bit 1 - Write Almost Complete Interrupt Enable"]
    #[inline(always)]
    pub fn wralcmplt(&mut self) -> WRALCMPLT_W {
        WRALCMPLT_W { w: self }
    }
    #[doc = "Bit 2 - Command Fail Interrupt Enable"]
    #[inline(always)]
    pub fn cmdfail(&mut self) -> CMDFAIL_W {
        CMDFAIL_W { w: self }
    }
    #[doc = "Bits 6:7 - Control 2-bit ECC Error Events"]
    #[inline(always)]
    pub fn ecc_error(&mut self) -> ECC_ERROR_W {
        ECC_ERROR_W { w: self }
    }
}
