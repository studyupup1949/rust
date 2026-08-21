#[doc = "Reader of register CMD"]
pub type R = crate::R<u32, super::CMD>;
#[doc = "Writer for register CMD"]
pub type W = crate::W<u32, super::CMD>;
#[doc = "Register CMD `reset()`'s with value 0"]
impl crate::ResetValue for super::CMD {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Commands\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum VALUE_A {
    #[doc = "0: IDLE"]
    IDLE = 0,
    #[doc = "1: ABORT"]
    ABORT = 1,
    #[doc = "2: Requests flash to enter Sleep mode"]
    SLEEP = 2,
    #[doc = "3: SIGN"]
    SIGN = 3,
    #[doc = "4: WRITE"]
    WRITE = 4,
    #[doc = "5: Checks all of User Space; fails if any bits in user space are cleared"]
    BLANK_CHECK = 5,
    #[doc = "6: ERASEPAGE"]
    ERASEPAGE = 6,
    #[doc = "7: MASSERASE"]
    MASSERASE = 7,
}
impl From<VALUE_A> for u8 {
    #[inline(always)]
    fn from(variant: VALUE_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, VALUE_A>;
impl VALUE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, VALUE_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(VALUE_A::IDLE),
            1 => Val(VALUE_A::ABORT),
            2 => Val(VALUE_A::SLEEP),
            3 => Val(VALUE_A::SIGN),
            4 => Val(VALUE_A::WRITE),
            5 => Val(VALUE_A::BLANK_CHECK),
            6 => Val(VALUE_A::ERASEPAGE),
            7 => Val(VALUE_A::MASSERASE),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `IDLE`"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == VALUE_A::IDLE
    }
    #[doc = "Checks if the value of the field is `ABORT`"]
    #[inline(always)]
    pub fn is_abort(&self) -> bool {
        *self == VALUE_A::ABORT
    }
    #[doc = "Checks if the value of the field is `SLEEP`"]
    #[inline(always)]
    pub fn is_sleep(&self) -> bool {
        *self == VALUE_A::SLEEP
    }
    #[doc = "Checks if the value of the field is `SIGN`"]
    #[inline(always)]
    pub fn is_sign(&self) -> bool {
        *self == VALUE_A::SIGN
    }
    #[doc = "Checks if the value of the field is `WRITE`"]
    #[inline(always)]
    pub fn is_write(&self) -> bool {
        *self == VALUE_A::WRITE
    }
    #[doc = "Checks if the value of the field is `BLANK_CHECK`"]
    #[inline(always)]
    pub fn is_blank_check(&self) -> bool {
        *self == VALUE_A::BLANK_CHECK
    }
    #[doc = "Checks if the value of the field is `ERASEPAGE`"]
    #[inline(always)]
    pub fn is_erasepage(&self) -> bool {
        *self == VALUE_A::ERASEPAGE
    }
    #[doc = "Checks if the value of the field is `MASSERASE`"]
    #[inline(always)]
    pub fn is_masserase(&self) -> bool {
        *self == VALUE_A::MASSERASE
    }
}
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: VALUE_A) -> &'a mut W {
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "IDLE"]
    #[inline(always)]
    pub fn idle(self) -> &'a mut W {
        self.variant(VALUE_A::IDLE)
    }
    #[doc = "ABORT"]
    #[inline(always)]
    pub fn abort(self) -> &'a mut W {
        self.variant(VALUE_A::ABORT)
    }
    #[doc = "Requests flash to enter Sleep mode"]
    #[inline(always)]
    pub fn sleep(self) -> &'a mut W {
        self.variant(VALUE_A::SLEEP)
    }
    #[doc = "SIGN"]
    #[inline(always)]
    pub fn sign(self) -> &'a mut W {
        self.variant(VALUE_A::SIGN)
    }
    #[doc = "WRITE"]
    #[inline(always)]
    pub fn write(self) -> &'a mut W {
        self.variant(VALUE_A::WRITE)
    }
    #[doc = "Checks all of User Space; fails if any bits in user space are cleared"]
    #[inline(always)]
    pub fn blank_check(self) -> &'a mut W {
        self.variant(VALUE_A::BLANK_CHECK)
    }
    #[doc = "ERASEPAGE"]
    #[inline(always)]
    pub fn erasepage(self) -> &'a mut W {
        self.variant(VALUE_A::ERASEPAGE)
    }
    #[doc = "MASSERASE"]
    #[inline(always)]
    pub fn masserase(self) -> &'a mut W {
        self.variant(VALUE_A::MASSERASE)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u32) & 0x0f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:3 - Commands"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Commands"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
