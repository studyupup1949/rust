#![no_std]

use core::ops::Range;
use embedded_hal::digital::{OutputPin, PinState};
use wrappers::{ShifterPin, ShifterValue, ShifterValueRange};

pub mod wrappers;

pub struct AdvancedShiftRegister<const N: usize, OP: OutputPin> {
    /// Shifter data (currently set bits)
    pub shifters: [u8; N],

    /// Data pin
    data_pin: OP,

    /// Clock pin
    clk_pin: OP,

    /// Latch pin
    latch_pin: OP,
}

impl<const N: usize, OP: OutputPin> AdvancedShiftRegister<N, OP> {
    pub fn new(data_pin: OP, clk_pin: OP, latch_pin: OP, default_val: u8) -> Self {
        Self {
            shifters: [default_val; N],
            data_pin,
            clk_pin,
            latch_pin,
        }
    }

    /// Get wrapper for value of one shifter register (for easy modyfing)
    pub fn get_shifter_mut(&mut self, i: usize) -> ShifterValue {
        unsafe {
            ShifterValue {
                inner: core::ptr::addr_of_mut!(self.shifters[i]),
                update_shifters_ptr: UpdateShiftersFuncPtr::new(self),
            }
        }
    }

    /// Get wrapper for one bit of specifed shifter register (with embedded_hal digitalpin trait)
    pub fn get_pin_mut(&mut self, i: usize, bit: u8) -> ShifterPin {
        unsafe {
            ShifterPin {
                bit,
                inner: core::ptr::addr_of_mut!(self.shifters[i]),
                update_shifters_ptr: UpdateShiftersFuncPtr::new(self),
            }
        }
    }

    /// Get wrapper for range of shifter registers
    /// (for easy modyfing of multiple registers at the same time)
    pub fn get_shifter_range_mut(&mut self, range: Range<usize>) -> ShifterValueRange {
        unsafe {
            ShifterValueRange {
                //len: range.len(),
                inner: core::ptr::addr_of_mut!(self.shifters[range]),
                update_shifters_ptr: UpdateShiftersFuncPtr::new(self),
            }
        }
    }

    /// Push stored shifters data onto shifter registers
    pub fn update_shifters(&mut self) {
        for i in (0..N).rev() {
            let mut val = self.shifters[i];

            for _ in 0..8 {
                let state = PinState::from(val & 1 > 0);
                _ = self.data_pin.set_state(state);
                val >>= 1;

                _ = self.clk_pin.set_high();
                _ = self.clk_pin.set_low();
            }
        }

        _ = self.latch_pin.set_high();
        _ = self.latch_pin.set_low();
    }

    /// Trampoline function to execute `.update_shifters()` function outside this struct
    /// without using any generics
    unsafe extern "C" fn update_shifters_trampoline(this: *mut AdvancedShiftRegister<N, OP>) {
        (&mut *this).update_shifters();
    }
}

#[derive(Clone)]
struct UpdateShiftersFuncPtr {
    parent: *mut (),
    update_shifters_ptr: unsafe extern "C" fn(*mut ()),
}

impl UpdateShiftersFuncPtr {
    pub unsafe fn new<const N: usize, OP: OutputPin>(
        parent: &mut AdvancedShiftRegister<N, OP>,
    ) -> Self {
        Self {
            parent: parent as *mut _ as *mut (),
            update_shifters_ptr: core::mem::transmute(
                AdvancedShiftRegister::update_shifters_trampoline
                    as unsafe extern "C" fn(*mut AdvancedShiftRegister<N, OP>),
            ),
        }
    }

    pub unsafe fn call_update_shifters(&self) {
        (self.update_shifters_ptr)(self.parent);
    }
}
