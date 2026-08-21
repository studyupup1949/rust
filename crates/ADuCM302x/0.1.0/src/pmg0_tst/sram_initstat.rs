#[doc = "Reader of register SRAM_INITSTAT"]
pub type R = crate::R<u32, super::SRAM_INITSTAT>;
#[doc = "Reader of field `BNK0`"]
pub type BNK0_R = crate::R<bool, bool>;
#[doc = "Reader of field `BNK1`"]
pub type BNK1_R = crate::R<bool, bool>;
#[doc = "Reader of field `BNK2`"]
pub type BNK2_R = crate::R<bool, bool>;
#[doc = "Reader of field `BNK3`"]
pub type BNK3_R = crate::R<bool, bool>;
#[doc = "Reader of field `BNK4`"]
pub type BNK4_R = crate::R<bool, bool>;
#[doc = "Reader of field `BNK5`"]
pub type BNK5_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Initialization Done of SRAM Bank 0"]
    #[inline(always)]
    pub fn bnk0(&self) -> BNK0_R {
        BNK0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Initialization Done of SRAM Bank 1"]
    #[inline(always)]
    pub fn bnk1(&self) -> BNK1_R {
        BNK1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Initialization Done of SRAM Bank 2"]
    #[inline(always)]
    pub fn bnk2(&self) -> BNK2_R {
        BNK2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Initialization Done of SRAM Bank 3"]
    #[inline(always)]
    pub fn bnk3(&self) -> BNK3_R {
        BNK3_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Initialization Done of SRAM Bank 4"]
    #[inline(always)]
    pub fn bnk4(&self) -> BNK4_R {
        BNK4_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Initialization Done of SRAM Bank 5"]
    #[inline(always)]
    pub fn bnk5(&self) -> BNK5_R {
        BNK5_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
