use crate::MutFuncPtr;
use core::borrow::BorrowMut;

#[derive(Clone)]
pub struct ShifterValue {
    /// Inner mutable byte value reference
    pub(crate) inner: *mut u8,

    /// Pointer to `.update_shifters()` function of AdvancedShiftRegister
    pub(crate) update_shifters_ptr: MutFuncPtr,
}

impl ShifterValue {
    /// Set value of shifter register and update shifters
    pub fn set_value(&self, value: u8) {
        unsafe {
            *self.inner = value;
            self.update_shifters_ptr.call();
        }
    }

    /// Get mutable reference to inner value (for easy bitwise operations)
    /// This function returns guard to value, automatically updates shifters after change
    pub fn value<'a>(&self) -> ShifterGuard<'a, u8> {
        unsafe {
            ShifterGuard {
                inner: self.inner.as_mut().unwrap(),
                update_shifters_ptr: self.update_shifters_ptr.clone(),
            }
        }
    }

    /// Push stored shifters data onto shifter registers
    pub fn update_shifters(&self) {
        unsafe {
            self.update_shifters_ptr.call();
        }
    }

    /// Get wrapper for one bit of shifter register (with embedded_hal digitalpin trait)
    pub fn get_pin_mut(&self, bit: u8, auto_shift: bool) -> ShifterPin {
        ShifterPin {
            bit,
            auto_update: auto_shift,
            inner: self.inner,
            update_shifters_ptr: self.update_shifters_ptr.clone(),
        }
    }
}

pub struct ShifterPin {
    /// Bit index to modify
    pub(crate) bit: u8,

    /// If after change should auto shift
    pub(crate) auto_update: bool,

    /// Inner mutable byte value reference
    pub(crate) inner: *mut u8,

    /// Pointer to `.update_shifters()` function of AdvancedShiftRegister
    pub(crate) update_shifters_ptr: MutFuncPtr,
}

impl embedded_hal::digital::ErrorType for ShifterPin {
    type Error = embedded_hal::digital::ErrorKind;
}

impl embedded_hal::digital::OutputPin for ShifterPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            *self.inner &= !(1 << (7 - self.bit));

            if self.auto_update {
                self.update_shifters_ptr.call();
            }
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            *self.inner |= 1 << (7 - self.bit);

            if self.auto_update {
                self.update_shifters_ptr.call();
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ShifterValueRange {
    /// Inner mutable byte array reference
    pub(crate) inner: *mut [u8],

    /// Len of array reference
    //pub(crate) len: usize,

    /// Pointer to `.update_shifters()` function of AdvancedShiftRegister
    pub(crate) update_shifters_ptr: MutFuncPtr,
}

impl ShifterValueRange {
    /// Set data of shifters and update them
    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr.copy_from_slice(data);
            self.update_shifters_ptr.call();
        }
    }

    /// Get mutable reference to inner array (for easy bitwise operations)
    /// This function returns guard to value, automatically updates shifters after change
    pub fn data<'a>(&self) -> ShifterGuard<'a, [u8]> {
        unsafe {
            ShifterGuard {
                inner: &mut *self.inner,
                update_shifters_ptr: self.update_shifters_ptr.clone(),
            }
        }
    }

    /// Set data of selected shifter and calls `.update_shifters()`
    pub fn set_value(&self, index: usize, value: u8) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index] = value;
            self.update_shifters_ptr.call();
        }
    }

    /// Get mutable reference to inner value of selected shifter (for easy bitwise operations)
    /// This function returns guard to value, automatically updates shifters after change
    pub fn value<'a>(&self, index: usize) -> ShifterGuard<'a, u8> {
        unsafe {
            let ptr = &mut *self.inner;

            ShifterGuard {
                inner: ptr[index].borrow_mut(),
                update_shifters_ptr: self.update_shifters_ptr.clone(),
            }
        }
    }

    /// Push stored shifters data onto shifter registers
    pub fn update_shifters(&self) {
        unsafe {
            self.update_shifters_ptr.call();
        }
    }
}

pub struct ShifterGuard<'a, T: ?Sized> {
    inner: &'a mut T,
    update_shifters_ptr: MutFuncPtr,
}

impl<'a, T: ?Sized> Drop for ShifterGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.update_shifters_ptr.call();
        }
    }
}

impl<'a, T: ?Sized> core::ops::Deref for ShifterGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T: ?Sized> core::ops::DerefMut for ShifterGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
