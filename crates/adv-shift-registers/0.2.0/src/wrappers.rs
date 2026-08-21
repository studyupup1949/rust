use core::borrow::BorrowMut;

use crate::UpdateShiftersFuncPtr;

#[derive(Clone)]
pub struct ShifterValue {
    /// Inner mutable byte value reference
    pub(crate) inner: *mut u8,

    /// Pointer to `.update_shifters()` function of AdvancedShiftRegister
    pub(crate) update_shifters_ptr: UpdateShiftersFuncPtr,
}

impl ShifterValue {
    /// Set value of shifter register and update shifters
    pub fn set_value(&self, value: u8) {
        unsafe {
            *self.inner = value;
            self.update_shifters_ptr.call_update_shifters();
        }
    }

    /// Get mutable reference to inner value (for easy bitwise operations)
    /// This function doesn't call `.update_shifters()`
    pub fn value<'a>(&self) -> &'a mut u8 {
        unsafe { self.inner.as_mut().unwrap() }
    }

    /// Push stored shifters data onto shifter registers
    pub fn update_shifters(&self) {
        unsafe {
            self.update_shifters_ptr.call_update_shifters();
        }
    }
}

pub struct ShifterPin {
    /// Bit index to modify
    pub(crate) bit: u8,

    /// Inner mutable byte value reference
    pub(crate) inner: *mut u8,

    /// Pointer to `.update_shifters()` function of AdvancedShiftRegister
    pub(crate) update_shifters_ptr: UpdateShiftersFuncPtr,
}

impl embedded_hal::digital::ErrorType for ShifterPin {
    type Error = embedded_hal::digital::ErrorKind;
}

impl embedded_hal::digital::OutputPin for ShifterPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            *self.inner &= !(1 << (7 - self.bit));
            self.update_shifters_ptr.call_update_shifters();
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            *self.inner |= 1 << (7 - self.bit);
            self.update_shifters_ptr.call_update_shifters();
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
    pub(crate) update_shifters_ptr: UpdateShiftersFuncPtr,
}

impl ShifterValueRange {
    /// Set data of shifters and update them
    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr.copy_from_slice(data);
            self.update_shifters_ptr.call_update_shifters();
        }
    }

    /// Get mutable reference to inner array (for easy bitwise operations)
    /// This function doesn't call `.update_shifters()`
    pub fn data<'a>(&self) -> &'a mut [u8] {
        unsafe { self.inner.as_mut().unwrap() }
    }

    /// Set data of selected shifter and update calls `.update_shifters()`
    pub fn set_value(&self, index: usize, value: u8) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index] = value;
            self.update_shifters_ptr.call_update_shifters();
        }
    }

    /// Get mutable reference to inner value of selected shifter (for easy bitwise operations)
    /// This function doesn't call `.update_shifters()`
    pub fn value<'a>(&self, index: usize) -> &'a mut u8 {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index].borrow_mut()
        }
    }

    /// Push stored shifters data onto shifter registers
    pub fn update_shifters(&self) {
        unsafe {
            self.update_shifters_ptr.call_update_shifters();
        }
    }
}
