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
        ShifterValue {
            inner: core::ptr::addr_of_mut!(self.shifters[i]),
            update_shifters_ptr: MutFuncPtr::new(self, Self::update_shifters_trampoline),
        }
    }

    /// Get wrapper for one bit of specifed shifter register (with embedded_hal digitalpin trait)
    pub fn get_pin_mut(&mut self, i: usize, bit: u8, auto_shift: bool) -> ShifterPin {
        ShifterPin {
            bit,
            auto_update: auto_shift,
            inner: core::ptr::addr_of_mut!(self.shifters[i]),
            update_shifters_ptr: MutFuncPtr::new(self, Self::update_shifters_trampoline),
        }
    }

    /// Get wrapper for range of shifter registers
    /// (for easy modyfing of multiple registers at the same time)
    pub fn get_shifter_range_mut(&mut self, range: Range<usize>) -> ShifterValueRange {
        ShifterValueRange {
            //len: range.len(),
            inner: core::ptr::addr_of_mut!(self.shifters[range]),
            update_shifters_ptr: MutFuncPtr::new(self, Self::update_shifters_trampoline),
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
    unsafe extern "C" fn update_shifters_trampoline(this: *mut Self) {
        (&mut *this).update_shifters();
    }
}

#[derive(Clone)]
struct MutFuncPtr {
    parent: *mut (),
    call_ptr: unsafe extern "C" fn(*mut ()),
}

impl MutFuncPtr {
    pub fn new<N>(parent: &mut N, function: unsafe extern "C" fn(*mut N)) -> Self {
        unsafe {
            Self {
                parent: parent as *mut _ as *mut (),
                call_ptr: core::mem::transmute(function),
            }
        }
    }

    pub unsafe fn call(&self) {
        (self.call_ptr)(self.parent);
    }
}
