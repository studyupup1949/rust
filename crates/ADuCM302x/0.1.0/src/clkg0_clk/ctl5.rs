#[doc = "Reader of register CTL5"]
pub type R = crate::R<u32, super::CTL5>;
#[doc = "Writer for register CTL5"]
pub type W = crate::W<u32, super::CTL5>;
#[doc = "Register CTL5 `reset()`'s with value 0x1f"]
impl crate::ResetValue for super::CTL5 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x1f
    }
}
#[doc = "Reader of field `GPTCLK0OFF`"]
pub type GPTCLK0OFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GPTCLK0OFF`"]
pub struct GPTCLK0OFF_W<'a> {
    w: &'a mut W,
}
impl<'a> GPTCLK0OFF_W<'a> {
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
#[doc = "Reader of field `GPTCLK1OFF`"]
pub type GPTCLK1OFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GPTCLK1OFF`"]
pub struct GPTCLK1OFF_W<'a> {
    w: &'a mut W,
}
impl<'a> GPTCLK1OFF_W<'a> {
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
#[doc = "Reader of field `GPTCLK2OFF`"]
pub type GPTCLK2OFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GPTCLK2OFF`"]
pub struct GPTCLK2OFF_W<'a> {
    w: &'a mut W,
}
impl<'a> GPTCLK2OFF_W<'a> {
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
#[doc = "Reader of field `UCLKI2COFF`"]
pub type UCLKI2COFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `UCLKI2COFF`"]
pub struct UCLKI2COFF_W<'a> {
    w: &'a mut W,
}
impl<'a> UCLKI2COFF_W<'a> {
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
#[doc = "Reader of field `GPIOCLKOFF`"]
pub type GPIOCLKOFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GPIOCLKOFF`"]
pub struct GPIOCLKOFF_W<'a> {
    w: &'a mut W,
}
impl<'a> GPIOCLKOFF_W<'a> {
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
#[doc = "Disables All Clocks Connected to All Peripherals\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PERCLKOFF_A {
    #[doc = "0: Clocks to all peripherals are active"]
    PERIPH_CLK_ACT = 0,
    #[doc = "1: Clocks to all peripherals are gated off"]
    PERIPH_CLK_OFF = 1,
}
impl From<PERCLKOFF_A> for bool {
    #[inline(always)]
    fn from(variant: PERCLKOFF_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PERCLKOFF`"]
pub type PERCLKOFF_R = crate::R<bool, PERCLKOFF_A>;
impl PERCLKOFF_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PERCLKOFF_A {
        match self.bits {
            false => PERCLKOFF_A::PERIPH_CLK_ACT,
            true => PERCLKOFF_A::PERIPH_CLK_OFF,
        }
    }
    #[doc = "Checks if the value of the field is `PERIPH_CLK_ACT`"]
    #[inline(always)]
    pub fn is_periph_clk_act(&self) -> bool {
        *self == PERCLKOFF_A::PERIPH_CLK_ACT
    }
    #[doc = "Checks if the value of the field is `PERIPH_CLK_OFF`"]
    #[inline(always)]
    pub fn is_periph_clk_off(&self) -> bool {
        *self == PERCLKOFF_A::PERIPH_CLK_OFF
    }
}
#[doc = "Write proxy for field `PERCLKOFF`"]
pub struct PERCLKOFF_W<'a> {
    w: &'a mut W,
}
impl<'a> PERCLKOFF_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PERCLKOFF_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Clocks to all peripherals are active"]
    #[inline(always)]
    pub fn periph_clk_act(self) -> &'a mut W {
        self.variant(PERCLKOFF_A::PERIPH_CLK_ACT)
    }
    #[doc = "Clocks to all peripherals are gated off"]
    #[inline(always)]
    pub fn periph_clk_off(self) -> &'a mut W {
        self.variant(PERCLKOFF_A::PERIPH_CLK_OFF)
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Timer 0 User Control"]
    #[inline(always)]
    pub fn gptclk0off(&self) -> GPTCLK0OFF_R {
        GPTCLK0OFF_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Timer 1 User Control"]
    #[inline(always)]
    pub fn gptclk1off(&self) -> GPTCLK1OFF_R {
        GPTCLK1OFF_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Timer 2 User Control"]
    #[inline(always)]
    pub fn gptclk2off(&self) -> GPTCLK2OFF_R {
        GPTCLK2OFF_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - I2C Clock User Control"]
    #[inline(always)]
    pub fn uclki2coff(&self) -> UCLKI2COFF_R {
        UCLKI2COFF_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - GPIO Clock Control"]
    #[inline(always)]
    pub fn gpioclkoff(&self) -> GPIOCLKOFF_R {
        GPIOCLKOFF_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Disables All Clocks Connected to All Peripherals"]
    #[inline(always)]
    pub fn perclkoff(&self) -> PERCLKOFF_R {
        PERCLKOFF_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Timer 0 User Control"]
    #[inline(always)]
    pub fn gptclk0off(&mut self) -> GPTCLK0OFF_W {
        GPTCLK0OFF_W { w: self }
    }
    #[doc = "Bit 1 - Timer 1 User Control"]
    #[inline(always)]
    pub fn gptclk1off(&mut self) -> GPTCLK1OFF_W {
        GPTCLK1OFF_W { w: self }
    }
    #[doc = "Bit 2 - Timer 2 User Control"]
    #[inline(always)]
    pub fn gptclk2off(&mut self) -> GPTCLK2OFF_W {
        GPTCLK2OFF_W { w: self }
    }
    #[doc = "Bit 3 - I2C Clock User Control"]
    #[inline(always)]
    pub fn uclki2coff(&mut self) -> UCLKI2COFF_W {
        UCLKI2COFF_W { w: self }
    }
    #[doc = "Bit 4 - GPIO Clock Control"]
    #[inline(always)]
    pub fn gpioclkoff(&mut self) -> GPIOCLKOFF_W {
        GPIOCLKOFF_W { w: self }
    }
    #[doc = "Bit 5 - Disables All Clocks Connected to All Peripherals"]
    #[inline(always)]
    pub fn perclkoff(&mut self) -> PERCLKOFF_W {
        PERCLKOFF_W { w: self }
    }
}
