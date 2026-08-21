#[doc = "Reader of register SRAM_CTL"]
pub type R = crate::R<u32, super::SRAM_CTL>;
#[doc = "Writer for register SRAM_CTL"]
pub type W = crate::W<u32, super::SRAM_CTL>;
#[doc = "Register SRAM_CTL `reset()`'s with value 0x8000_0000"]
impl crate::ResetValue for super::SRAM_CTL {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x8000_0000
    }
}
#[doc = "Reader of field `BNK0EN`"]
pub type BNK0EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK0EN`"]
pub struct BNK0EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK0EN_W<'a> {
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
#[doc = "Reader of field `BNK1EN`"]
pub type BNK1EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK1EN`"]
pub struct BNK1EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK1EN_W<'a> {
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
#[doc = "Reader of field `BNK2EN`"]
pub type BNK2EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK2EN`"]
pub struct BNK2EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK2EN_W<'a> {
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
#[doc = "Reader of field `BNK3EN`"]
pub type BNK3EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK3EN`"]
pub struct BNK3EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK3EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `BNK4EN`"]
pub type BNK4EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK4EN`"]
pub struct BNK4EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK4EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `BNK5EN`"]
pub type BNK5EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK5EN`"]
pub struct BNK5EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK5EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `STARTINIT`"]
pub type STARTINIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STARTINIT`"]
pub struct STARTINIT_W<'a> {
    w: &'a mut W,
}
impl<'a> STARTINIT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u32) & 0x01) << 13);
        self.w
    }
}
#[doc = "Reader of field `AUTOINIT`"]
pub type AUTOINIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AUTOINIT`"]
pub struct AUTOINIT_W<'a> {
    w: &'a mut W,
}
impl<'a> AUTOINIT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u32) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `ABTINIT`"]
pub type ABTINIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ABTINIT`"]
pub struct ABTINIT_W<'a> {
    w: &'a mut W,
}
impl<'a> ABTINIT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u32) & 0x01) << 15);
        self.w
    }
}
#[doc = "Reader of field `PENBNK0`"]
pub type PENBNK0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK0`"]
pub struct PENBNK0_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK0_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 16)) | (((value as u32) & 0x01) << 16);
        self.w
    }
}
#[doc = "Reader of field `PENBNK1`"]
pub type PENBNK1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK1`"]
pub struct PENBNK1_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK1_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 17)) | (((value as u32) & 0x01) << 17);
        self.w
    }
}
#[doc = "Reader of field `PENBNK2`"]
pub type PENBNK2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK2`"]
pub struct PENBNK2_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 18)) | (((value as u32) & 0x01) << 18);
        self.w
    }
}
#[doc = "Reader of field `PENBNK3`"]
pub type PENBNK3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK3`"]
pub struct PENBNK3_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK3_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 19)) | (((value as u32) & 0x01) << 19);
        self.w
    }
}
#[doc = "Reader of field `PENBNK4`"]
pub type PENBNK4_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK4`"]
pub struct PENBNK4_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK4_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 20)) | (((value as u32) & 0x01) << 20);
        self.w
    }
}
#[doc = "Reader of field `PENBNK5`"]
pub type PENBNK5_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PENBNK5`"]
pub struct PENBNK5_W<'a> {
    w: &'a mut W,
}
impl<'a> PENBNK5_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 21)) | (((value as u32) & 0x01) << 21);
        self.w
    }
}
#[doc = "Reader of field `INSTREN`"]
pub type INSTREN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INSTREN`"]
pub struct INSTREN_W<'a> {
    w: &'a mut W,
}
impl<'a> INSTREN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 31)) | (((value as u32) & 0x01) << 31);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable Initialization of SRAM Bank 0"]
    #[inline(always)]
    pub fn bnk0en(&self) -> BNK0EN_R {
        BNK0EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable Initialization of SRAM Bank 1"]
    #[inline(always)]
    pub fn bnk1en(&self) -> BNK1EN_R {
        BNK1EN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable Initialization of SRAM Bank 2"]
    #[inline(always)]
    pub fn bnk2en(&self) -> BNK2EN_R {
        BNK2EN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Enable Initialization of SRAM Bank 3"]
    #[inline(always)]
    pub fn bnk3en(&self) -> BNK3EN_R {
        BNK3EN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Enable Initialization of SRAM Bank 4"]
    #[inline(always)]
    pub fn bnk4en(&self) -> BNK4EN_R {
        BNK4EN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Enable Initialization of SRAM Bank 5"]
    #[inline(always)]
    pub fn bnk5en(&self) -> BNK5EN_R {
        BNK5EN_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Write 1 to Trigger Initialization"]
    #[inline(always)]
    pub fn startinit(&self) -> STARTINIT_R {
        STARTINIT_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Automatic Initialization on Wakeup from Hibernate Mode"]
    #[inline(always)]
    pub fn autoinit(&self) -> AUTOINIT_R {
        AUTOINIT_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Abort Current Initialization. Self-cleared"]
    #[inline(always)]
    pub fn abtinit(&self) -> ABTINIT_R {
        ABTINIT_R::new(((self.bits >> 15) & 0x01) != 0)
    }
    #[doc = "Bit 16 - Enable Parity Check SRAM Bank 0"]
    #[inline(always)]
    pub fn penbnk0(&self) -> PENBNK0_R {
        PENBNK0_R::new(((self.bits >> 16) & 0x01) != 0)
    }
    #[doc = "Bit 17 - Enable Parity Check SRAM Bank 1"]
    #[inline(always)]
    pub fn penbnk1(&self) -> PENBNK1_R {
        PENBNK1_R::new(((self.bits >> 17) & 0x01) != 0)
    }
    #[doc = "Bit 18 - Enable Parity Check SRAM Bank 2"]
    #[inline(always)]
    pub fn penbnk2(&self) -> PENBNK2_R {
        PENBNK2_R::new(((self.bits >> 18) & 0x01) != 0)
    }
    #[doc = "Bit 19 - Enable Parity Check SRAM Bank 3"]
    #[inline(always)]
    pub fn penbnk3(&self) -> PENBNK3_R {
        PENBNK3_R::new(((self.bits >> 19) & 0x01) != 0)
    }
    #[doc = "Bit 20 - Enable Parity Check SRAM Bank 4"]
    #[inline(always)]
    pub fn penbnk4(&self) -> PENBNK4_R {
        PENBNK4_R::new(((self.bits >> 20) & 0x01) != 0)
    }
    #[doc = "Bit 21 - Enable Parity Check SRAM Bank 5"]
    #[inline(always)]
    pub fn penbnk5(&self) -> PENBNK5_R {
        PENBNK5_R::new(((self.bits >> 21) & 0x01) != 0)
    }
    #[doc = "Bit 31 - Enables Instruction SRAM"]
    #[inline(always)]
    pub fn instren(&self) -> INSTREN_R {
        INSTREN_R::new(((self.bits >> 31) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Initialization of SRAM Bank 0"]
    #[inline(always)]
    pub fn bnk0en(&mut self) -> BNK0EN_W {
        BNK0EN_W { w: self }
    }
    #[doc = "Bit 1 - Enable Initialization of SRAM Bank 1"]
    #[inline(always)]
    pub fn bnk1en(&mut self) -> BNK1EN_W {
        BNK1EN_W { w: self }
    }
    #[doc = "Bit 2 - Enable Initialization of SRAM Bank 2"]
    #[inline(always)]
    pub fn bnk2en(&mut self) -> BNK2EN_W {
        BNK2EN_W { w: self }
    }
    #[doc = "Bit 3 - Enable Initialization of SRAM Bank 3"]
    #[inline(always)]
    pub fn bnk3en(&mut self) -> BNK3EN_W {
        BNK3EN_W { w: self }
    }
    #[doc = "Bit 4 - Enable Initialization of SRAM Bank 4"]
    #[inline(always)]
    pub fn bnk4en(&mut self) -> BNK4EN_W {
        BNK4EN_W { w: self }
    }
    #[doc = "Bit 5 - Enable Initialization of SRAM Bank 5"]
    #[inline(always)]
    pub fn bnk5en(&mut self) -> BNK5EN_W {
        BNK5EN_W { w: self }
    }
    #[doc = "Bit 13 - Write 1 to Trigger Initialization"]
    #[inline(always)]
    pub fn startinit(&mut self) -> STARTINIT_W {
        STARTINIT_W { w: self }
    }
    #[doc = "Bit 14 - Automatic Initialization on Wakeup from Hibernate Mode"]
    #[inline(always)]
    pub fn autoinit(&mut self) -> AUTOINIT_W {
        AUTOINIT_W { w: self }
    }
    #[doc = "Bit 15 - Abort Current Initialization. Self-cleared"]
    #[inline(always)]
    pub fn abtinit(&mut self) -> ABTINIT_W {
        ABTINIT_W { w: self }
    }
    #[doc = "Bit 16 - Enable Parity Check SRAM Bank 0"]
    #[inline(always)]
    pub fn penbnk0(&mut self) -> PENBNK0_W {
        PENBNK0_W { w: self }
    }
    #[doc = "Bit 17 - Enable Parity Check SRAM Bank 1"]
    #[inline(always)]
    pub fn penbnk1(&mut self) -> PENBNK1_W {
        PENBNK1_W { w: self }
    }
    #[doc = "Bit 18 - Enable Parity Check SRAM Bank 2"]
    #[inline(always)]
    pub fn penbnk2(&mut self) -> PENBNK2_W {
        PENBNK2_W { w: self }
    }
    #[doc = "Bit 19 - Enable Parity Check SRAM Bank 3"]
    #[inline(always)]
    pub fn penbnk3(&mut self) -> PENBNK3_W {
        PENBNK3_W { w: self }
    }
    #[doc = "Bit 20 - Enable Parity Check SRAM Bank 4"]
    #[inline(always)]
    pub fn penbnk4(&mut self) -> PENBNK4_W {
        PENBNK4_W { w: self }
    }
    #[doc = "Bit 21 - Enable Parity Check SRAM Bank 5"]
    #[inline(always)]
    pub fn penbnk5(&mut self) -> PENBNK5_W {
        PENBNK5_W { w: self }
    }
    #[doc = "Bit 31 - Enables Instruction SRAM"]
    #[inline(always)]
    pub fn instren(&mut self) -> INSTREN_W {
        INSTREN_W { w: self }
    }
}
