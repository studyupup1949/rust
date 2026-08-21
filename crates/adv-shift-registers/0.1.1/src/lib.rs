#![no_std]

use core::{borrow::BorrowMut, ops::Range};
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

    pub fn get_shifter_mut(&mut self, i: usize) -> ShifterValue {
        ShifterValue {
            inner: core::ptr::addr_of_mut!(self.shifters[i]),
        }
    }

    pub fn get_shifter_range_mut(&mut self, range: Range<usize>) -> ShifterValueRange {
        ShifterValueRange {
            //len: range.len(),
            inner: core::ptr::addr_of_mut!(self.shifters[range]),
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
pub struct ShifterValue {
    inner: *mut u8,
}

impl ShifterValue {
    pub fn set_value(&self, value: u8) {
        unsafe {
            *self.inner = value;
        }
    }

    pub fn value<'a>(&self) -> &'a mut u8 {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

#[derive(Clone)]
pub struct ShifterValueRange {
    inner: *mut [u8],
    //len: usize,
}

impl ShifterValueRange {
    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr.copy_from_slice(data);
        }
    }

    pub fn data<'a>(&self) -> &'a mut [u8] {
        unsafe { self.inner.as_mut().unwrap() }
    }

    pub fn set_value(&self, index: usize, value: u8) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index] = value;
        }
    }

    pub fn value<'a>(&self, index: usize) -> &'a mut u8 {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index].borrow_mut()
        }
    }
}
