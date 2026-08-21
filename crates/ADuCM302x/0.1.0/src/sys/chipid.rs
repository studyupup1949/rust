#[doc = "Reader of register CHIPID"]
pub type R = crate::R<u16, super::CHIPID>;
#[doc = "Reader of field `REV`"]
pub type REV_R = crate::R<u8, u8>;
#[doc = "Reader of field `PARTID`"]
pub type PARTID_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:3 - Silicon Revision"]
    #[inline(always)]
    pub fn rev(&self) -> REV_R {
        REV_R::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:15 - Part Identifier"]
    #[inline(always)]
    pub fn partid(&self) -> PARTID_R {
        PARTID_R::new(((self.bits >> 4) & 0x0fff) as u16)
    }
}
