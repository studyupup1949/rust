#[doc = "Reader of register ECC_ADDR"]
pub type R = crate::R<u32, super::ECC_ADDR>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:18 - ECC Error Address"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x0007_ffff) as u32)
    }
}
