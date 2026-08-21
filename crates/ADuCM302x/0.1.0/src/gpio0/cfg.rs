#[doc = "Reader of register CFG"]
pub type R = crate::R<u32, super::CFG>;
#[doc = "Writer for register CFG"]
pub type W = crate::W<u32, super::CFG>;
#[doc = "Register CFG `reset()`'s with value 0"]
impl crate::ResetValue for super::CFG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `PIN00`"]
pub type PIN00_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN00`"]
pub struct PIN00_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN00_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `PIN01`"]
pub type PIN01_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN01`"]
pub struct PIN01_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN01_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 2)) | (((value as u32) & 0x03) << 2);
        self.w
    }
}
#[doc = "Reader of field `PIN02`"]
pub type PIN02_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN02`"]
pub struct PIN02_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN02_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 4)) | (((value as u32) & 0x03) << 4);
        self.w
    }
}
#[doc = "Reader of field `PIN03`"]
pub type PIN03_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN03`"]
pub struct PIN03_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN03_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 6)) | (((value as u32) & 0x03) << 6);
        self.w
    }
}
#[doc = "Reader of field `PIN04`"]
pub type PIN04_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN04`"]
pub struct PIN04_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN04_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 8)) | (((value as u32) & 0x03) << 8);
        self.w
    }
}
#[doc = "Reader of field `PIN05`"]
pub type PIN05_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN05`"]
pub struct PIN05_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN05_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 10)) | (((value as u32) & 0x03) << 10);
        self.w
    }
}
#[doc = "Reader of field `PIN06`"]
pub type PIN06_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN06`"]
pub struct PIN06_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN06_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 12)) | (((value as u32) & 0x03) << 12);
        self.w
    }
}
#[doc = "Reader of field `PIN07`"]
pub type PIN07_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN07`"]
pub struct PIN07_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN07_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 14)) | (((value as u32) & 0x03) << 14);
        self.w
    }
}
#[doc = "Reader of field `PIN08`"]
pub type PIN08_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN08`"]
pub struct PIN08_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN08_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 16)) | (((value as u32) & 0x03) << 16);
        self.w
    }
}
#[doc = "Reader of field `PIN09`"]
pub type PIN09_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN09`"]
pub struct PIN09_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN09_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 18)) | (((value as u32) & 0x03) << 18);
        self.w
    }
}
#[doc = "Reader of field `PIN10`"]
pub type PIN10_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN10`"]
pub struct PIN10_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN10_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 20)) | (((value as u32) & 0x03) << 20);
        self.w
    }
}
#[doc = "Reader of field `PIN11`"]
pub type PIN11_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN11`"]
pub struct PIN11_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN11_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 22)) | (((value as u32) & 0x03) << 22);
        self.w
    }
}
#[doc = "Reader of field `PIN12`"]
pub type PIN12_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN12`"]
pub struct PIN12_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN12_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 24)) | (((value as u32) & 0x03) << 24);
        self.w
    }
}
#[doc = "Reader of field `PIN13`"]
pub type PIN13_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN13`"]
pub struct PIN13_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN13_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 26)) | (((value as u32) & 0x03) << 26);
        self.w
    }
}
#[doc = "Reader of field `PIN14`"]
pub type PIN14_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN14`"]
pub struct PIN14_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN14_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 28)) | (((value as u32) & 0x03) << 28);
        self.w
    }
}
#[doc = "Reader of field `PIN15`"]
pub type PIN15_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PIN15`"]
pub struct PIN15_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN15_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 30)) | (((value as u32) & 0x03) << 30);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Pin 0 Configuration Bits"]
    #[inline(always)]
    pub fn pin00(&self) -> PIN00_R {
        PIN00_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 2:3 - Pin 1 Configuration Bits"]
    #[inline(always)]
    pub fn pin01(&self) -> PIN01_R {
        PIN01_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bits 4:5 - Pin 2 Configuration Bits"]
    #[inline(always)]
    pub fn pin02(&self) -> PIN02_R {
        PIN02_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 6:7 - Pin 3 Configuration Bits"]
    #[inline(always)]
    pub fn pin03(&self) -> PIN03_R {
        PIN03_R::new(((self.bits >> 6) & 0x03) as u8)
    }
    #[doc = "Bits 8:9 - Pin 4 Configuration Bits"]
    #[inline(always)]
    pub fn pin04(&self) -> PIN04_R {
        PIN04_R::new(((self.bits >> 8) & 0x03) as u8)
    }
    #[doc = "Bits 10:11 - Pin 5 Configuration Bits"]
    #[inline(always)]
    pub fn pin05(&self) -> PIN05_R {
        PIN05_R::new(((self.bits >> 10) & 0x03) as u8)
    }
    #[doc = "Bits 12:13 - Pin 6 Configuration Bits"]
    #[inline(always)]
    pub fn pin06(&self) -> PIN06_R {
        PIN06_R::new(((self.bits >> 12) & 0x03) as u8)
    }
    #[doc = "Bits 14:15 - Pin 7 Configuration Bits"]
    #[inline(always)]
    pub fn pin07(&self) -> PIN07_R {
        PIN07_R::new(((self.bits >> 14) & 0x03) as u8)
    }
    #[doc = "Bits 16:17 - Pin 8 Configuration Bits"]
    #[inline(always)]
    pub fn pin08(&self) -> PIN08_R {
        PIN08_R::new(((self.bits >> 16) & 0x03) as u8)
    }
    #[doc = "Bits 18:19 - Pin 9 Configuration Bits"]
    #[inline(always)]
    pub fn pin09(&self) -> PIN09_R {
        PIN09_R::new(((self.bits >> 18) & 0x03) as u8)
    }
    #[doc = "Bits 20:21 - Pin 10 Configuration Bits"]
    #[inline(always)]
    pub fn pin10(&self) -> PIN10_R {
        PIN10_R::new(((self.bits >> 20) & 0x03) as u8)
    }
    #[doc = "Bits 22:23 - Pin 11 Configuration Bits"]
    #[inline(always)]
    pub fn pin11(&self) -> PIN11_R {
        PIN11_R::new(((self.bits >> 22) & 0x03) as u8)
    }
    #[doc = "Bits 24:25 - Pin 12 Configuration Bits"]
    #[inline(always)]
    pub fn pin12(&self) -> PIN12_R {
        PIN12_R::new(((self.bits >> 24) & 0x03) as u8)
    }
    #[doc = "Bits 26:27 - Pin 13 Configuration Bits"]
    #[inline(always)]
    pub fn pin13(&self) -> PIN13_R {
        PIN13_R::new(((self.bits >> 26) & 0x03) as u8)
    }
    #[doc = "Bits 28:29 - Pin 14 Configuration Bits"]
    #[inline(always)]
    pub fn pin14(&self) -> PIN14_R {
        PIN14_R::new(((self.bits >> 28) & 0x03) as u8)
    }
    #[doc = "Bits 30:31 - Pin 15 Configuration Bits"]
    #[inline(always)]
    pub fn pin15(&self) -> PIN15_R {
        PIN15_R::new(((self.bits >> 30) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Pin 0 Configuration Bits"]
    #[inline(always)]
    pub fn pin00(&mut self) -> PIN00_W {
        PIN00_W { w: self }
    }
    #[doc = "Bits 2:3 - Pin 1 Configuration Bits"]
    #[inline(always)]
    pub fn pin01(&mut self) -> PIN01_W {
        PIN01_W { w: self }
    }
    #[doc = "Bits 4:5 - Pin 2 Configuration Bits"]
    #[inline(always)]
    pub fn pin02(&mut self) -> PIN02_W {
        PIN02_W { w: self }
    }
    #[doc = "Bits 6:7 - Pin 3 Configuration Bits"]
    #[inline(always)]
    pub fn pin03(&mut self) -> PIN03_W {
        PIN03_W { w: self }
    }
    #[doc = "Bits 8:9 - Pin 4 Configuration Bits"]
    #[inline(always)]
    pub fn pin04(&mut self) -> PIN04_W {
        PIN04_W { w: self }
    }
    #[doc = "Bits 10:11 - Pin 5 Configuration Bits"]
    #[inline(always)]
    pub fn pin05(&mut self) -> PIN05_W {
        PIN05_W { w: self }
    }
    #[doc = "Bits 12:13 - Pin 6 Configuration Bits"]
    #[inline(always)]
    pub fn pin06(&mut self) -> PIN06_W {
        PIN06_W { w: self }
    }
    #[doc = "Bits 14:15 - Pin 7 Configuration Bits"]
    #[inline(always)]
    pub fn pin07(&mut self) -> PIN07_W {
        PIN07_W { w: self }
    }
    #[doc = "Bits 16:17 - Pin 8 Configuration Bits"]
    #[inline(always)]
    pub fn pin08(&mut self) -> PIN08_W {
        PIN08_W { w: self }
    }
    #[doc = "Bits 18:19 - Pin 9 Configuration Bits"]
    #[inline(always)]
    pub fn pin09(&mut self) -> PIN09_W {
        PIN09_W { w: self }
    }
    #[doc = "Bits 20:21 - Pin 10 Configuration Bits"]
    #[inline(always)]
    pub fn pin10(&mut self) -> PIN10_W {
        PIN10_W { w: self }
    }
    #[doc = "Bits 22:23 - Pin 11 Configuration Bits"]
    #[inline(always)]
    pub fn pin11(&mut self) -> PIN11_W {
        PIN11_W { w: self }
    }
    #[doc = "Bits 24:25 - Pin 12 Configuration Bits"]
    #[inline(always)]
    pub fn pin12(&mut self) -> PIN12_W {
        PIN12_W { w: self }
    }
    #[doc = "Bits 26:27 - Pin 13 Configuration Bits"]
    #[inline(always)]
    pub fn pin13(&mut self) -> PIN13_W {
        PIN13_W { w: self }
    }
    #[doc = "Bits 28:29 - Pin 14 Configuration Bits"]
    #[inline(always)]
    pub fn pin14(&mut self) -> PIN14_W {
        PIN14_W { w: self }
    }
    #[doc = "Bits 30:31 - Pin 15 Configuration Bits"]
    #[inline(always)]
    pub fn pin15(&mut self) -> PIN15_W {
        PIN15_W { w: self }
    }
}
