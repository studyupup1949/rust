#![no_std]

use core::ops::Range;
use embedded_hal::digital::{OutputPin, PinState};

pub struct AdvancedShiftRegister<const N: usize, OP: OutputPin> {
    pub shifters: [u8; N],
    data_pin: OP,
    clk_pin: OP,
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

    pub fn get_shifter_mut(&mut self, i: usize) -> ShifterValue<N, OP> {
        ShifterValue {
            inner: core::ptr::addr_of_mut!(self.shifters[i]),
            parent: core::ptr::addr_of_mut!(*self),
        }
    }

    pub fn get_shifter_range_mut(&mut self, range: Range<usize>) -> ShifterValueRange<N, OP> {
        ShifterValueRange {
            //len: range.len(),
            inner: core::ptr::addr_of_mut!(self.shifters[range]),
            parent: core::ptr::addr_of_mut!(*self),
        }
    }

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
}

#[derive(Clone)]
pub struct ShifterValue<const N: usize, OP: OutputPin> {
    inner: *mut u8,
    parent: *mut AdvancedShiftRegister<N, OP>,
}

impl<const N: usize, OP: OutputPin> ShifterValue<N, OP> {
    pub fn set_value(&self, value: u8) {
        unsafe {
            *self.inner = value;
        }
    }

    pub fn update_shifters(&self) {
        unsafe {
            (*self.parent).update_shifters();
        }
    }
}

#[derive(Clone)]
pub struct ShifterValueRange<const N: usize, OP: OutputPin> {
    inner: *mut [u8],
    //len: usize,
    parent: *mut AdvancedShiftRegister<N, OP>,
}

impl<const N: usize, OP: OutputPin> ShifterValueRange<N, OP> {
    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr.copy_from_slice(data);
        }
    }

    pub fn set_value(&self, index: usize, value: u8) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index] = value;
        }
    }

    pub fn update_shifters(&self) {
        unsafe {
            (*self.parent).update_shifters();
        }
    }
}
