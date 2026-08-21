//! Generic Registry Helper
//!
//! For implementing register maps for various I2C/SPI devices
//!
//!  For max 4 byte long registers
//!  Inteprets data though u32's
//!
//! Example:
//!
//! ```
//! use super::reg_helper::*;
//!
//! ...
//!
//! // General Flag Field ID
//! #[derive(Debug, PartialEq)]
//! pub enum FL {}
//!
//! pub const DEVID: Register<FL> = validated(Register {
//!     addr:   0x00,
//!     access: Mode::R,
//!     bytes:  1,
//!     order:  End::Little,
//!     fields: None,
//! });
//!
//! #[derive(Debug, PartialEq)]
//! pub enum FL_TA {
//!     SUPPRESS,
//!     TAP_X_EN,
//!     TAP_Y_EN,
//!     TAP_Z_EN,
//! }
//!
//! pub const TAP_AXES: Register<FL_TA> = validated(Register {
//!     addr:   0x2A,
//!     access: Mode::RW,
//!     bytes:  1,
//!     order:  End::Little,
//!     fields: Some(&[
//!         Field {
//!             id:     FL_TA::SUPPRESS,
//!             width:  1,
//!             offset: 3,
//!         },
//!         Field {
//!             id:     FL_TA::TAP_X_EN,
//!             width:  1,
//!             offset: 2,
//!         },
//!         Field {
//!             id:     FL_TA::TAP_Y_EN,
//!             width:  1,
//!             offset: 1,
//!         },
//!         Field {
//!             id:     FL_TA::TAP_Z_EN,
//!             width:  1,
//!             offset: 0,
//!         },
//!     ]),
//! });
//!
//! ...
//!
//! let dev_id = DEVID::read(&mut bus).unwrap();
//! let suppress = TAP_AXES.read_field(&mut bus, FL_TA::SUPPRESS).unwrap();;
//! TAP_AXES::write_field(&mut bus, FL_TA::TAP_X_EN, 0x1).unwrap();
//!  
//! ```

#![allow(non_camel_case_types)]

use core::fmt::{Debug, Display};
use core::result::Result;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

const MAX_BYTES: u8 = 4;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum End {
    Little,
    Big,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum Mode {
    R,
    W,
    RW,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Data
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Field definition for a partitioned register
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Field<ENUM> {
    pub id:     ENUM,
    pub width:  u8, // ie. 0b_0011_0000 has a width of 2
    pub offset: u8, // ie. 0b_0011_0000 has an offset of 4
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Register
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug)]
pub struct Register<ENUM: 'static + PartialEq> {
    pub addr:   u8,
    pub access: Mode,
    pub bytes:  u8,
    pub order:  End,
    pub fields: Option<&'static [Field<ENUM>]>,
}

impl<ENUM: PartialEq> Register<ENUM> {
    // —————————————————————————————————————————— Reads ————————————————————————————————————————————

    /// Read the registry, trim and format the data, then store it as u32.
    /// Endian conversion done based on the order specified in self.order
    pub fn read<B: RegisterBus>(&self, bus: &mut B) -> Result<u32, ErrorReg<B::Error>> {
        if matches!(self.access, Mode::W) {
            return Err(ErrorReg::WriteOnly);
        }

        // Read bytes
        let mut raw_bytes = [0u8; 4];
        bus.read_register(self.addr, &mut raw_bytes[..self.bytes as usize])?;

        // Trim and convert endian
        let result = match (self.bytes, &self.order) {
            (1, _) => raw_bytes[0] as u32,

            (2, End::Little) => u16::from_le_bytes([raw_bytes[0], raw_bytes[1]]) as u32,
            (2, End::Big) => u16::from_be_bytes([raw_bytes[0], raw_bytes[1]]) as u32,

            (3, End::Little) => u32::from_le_bytes([raw_bytes[0], raw_bytes[1], raw_bytes[2], 0]),
            (3, End::Big) => u32::from_be_bytes([0, raw_bytes[0], raw_bytes[1], raw_bytes[2]]),

            (4, End::Little) => u32::from_le_bytes(raw_bytes),
            (4, End::Big) => u32::from_be_bytes(raw_bytes),
            _ => unreachable!(),
        };

        Ok(result)
    }

    /// Read a single field (pre-filtered)
    /// Endian conversion done based based on the order specified in the data structure
    pub fn read_field<B: RegisterBus>(
        &self,
        bus: &mut B,
        field_id: ENUM,
    ) -> Result<u32, ErrorReg<B::Error>> {
        let fields = self.fields.ok_or(ErrorReg::NoFields)?;

        let field = fields
            .iter()
            .find(|f| f.id == field_id)
            .ok_or(ErrorReg::FieldNotFound)?;

        // Read bytes
        let raw = self.read(bus)?;
        let mask = create_mask(field.width, field.offset);
        let val = (raw & mask) >> field.offset;
        Ok(val)
    }

    /// Read multiple fields
    /// Endian conversion done based based on the order specified in the data structure
    pub fn read_multiple_fields<B: RegisterBus>(
        &self,
        bus: &mut B,
        field_ids: &[ENUM],
        output: &mut [u32],
    ) -> Result<(), ErrorReg<B::Error>> {
        if field_ids.len() > output.len() {
            return Err(ErrorReg::BufferOverrun);
        }
        let fields = self.fields.ok_or(ErrorReg::NoFields)?;

        // Read the register once
        let raw_read = self.read(bus)?;

        // Extract each field
        for (i, field_id) in field_ids.iter().enumerate() {
            let field = fields
                .iter()
                .find(|f| f.id == *field_id)
                .ok_or(ErrorReg::FieldNotFound)?;

            // Extract field value using mask and shift
            let mask = create_mask(field.width, field.offset);
            output[i] = (raw_read & mask) >> field.offset;
        }

        Ok(())
    }

    /// Read a memory block starting at this register address into the provided buffer
    pub fn read_raw_buffer<B: RegisterBus>(
        &self,
        bus: &mut B,
        buf: &mut [u8],
    ) -> Result<(), ErrorReg<B::Error>> {
        bus.read_register(self.addr, buf)?;
        Ok(())
    }

    // —————————————————————————————————————————— Writes ———————————————————————————————————————————

    /// Write a bit sequence (given as a u32 LE) to the register
    /// Endian conversion done internally based based on the order specified in the self.order
    pub fn write<B: RegisterBus>(&self, bus: &mut B, val: u32) -> Result<(), ErrorReg<B::Error>> {
        if !matches!(self.access, Mode::RW | Mode::W) {
            return Err(ErrorReg::ReadOnly);
        }

        let len = self.bytes as usize;
        let mut buf: [u8; 4] = val.to_le_bytes();

        // Reverse first N bytes if big-endian
        if matches!(self.order, End::Big) {
            buf[..len].reverse();
        }

        bus.write_register(self.addr, &buf[..len])?;
        Ok(())
    }

    /// Write the value of a single field
    /// Performs read-modify-write internally
    pub fn write_field<B: RegisterBus>(
        &self,
        bus: &mut B,
        field_id: ENUM,
        value: u32,
    ) -> Result<(), ErrorReg<B::Error>> {
        // write_field requires RW access because it performs read-modify-write
        if !matches!(self.access, Mode::RW) {
            return Err(ErrorReg::ReqRW);
        }

        let fields = self.fields.ok_or(ErrorReg::NoFields)?;

        let field = fields
            .iter()
            .find(|f| f.id == field_id)
            .ok_or(ErrorReg::FieldNotFound)?;

        // Validate value to field fit
        let max_field_val = if field.width == 32 { u32::MAX } else { (1u32 << field.width) - 1 };
        if value > max_field_val {
            return Err(ErrorReg::ValueTooLarge);
        }

        // Read-modify-write
        let current_val = self.read(bus)?;
        let mask = create_mask(field.width, field.offset);
        let new_val = (current_val & !mask) | ((value << field.offset) & mask);
        self.write(bus, new_val)
    }

    /// Write to multiple fields in one transaction
    /// Performs read-modify-write internally
    pub fn write_multiple_fields<B: RegisterBus>(
        &self,
        bus: &mut B,
        fields_with_values: &[(ENUM, u32)],
    ) -> Result<(), ErrorReg<B::Error>> {
        // write_field requires RW access because it performs read-modify-write
        if !matches!(self.access, Mode::RW) {
            return Err(ErrorReg::ReqRW);
        }

        let fields = self.fields.ok_or(ErrorReg::NoFields)?;

        let current_val = self.read(bus)?;
        let mut new_val = 0_u32;
        let mut new_mask = 0_u32;

        for (id, value) in fields_with_values {
            let field = fields
                .iter()
                .find(|f| f.id == *id)
                .ok_or(ErrorReg::FieldNotFound)?;

            // Validate value to field fit
            let max_field_val =
                if field.width == 32 { u32::MAX } else { (1u32 << field.width) - 1 };
            if *value > max_field_val {
                return Err(ErrorReg::ValueTooLarge);
            }

            // Read-modify-write
            let mask = create_mask(field.width, field.offset);
            // Appending value
            new_val |= (value << field.offset) & mask;
            // Appending mask
            new_mask |= mask;
        }

        // Computing and writing the final value
        new_val = (current_val & !new_mask) | (new_val & new_mask);
        self.write(bus, new_val)
    }

    /// Writing a memory block starting at this register address
    /// Data buffer needs to be pre-ordered for the desired Endian format
    #[inline]
    pub fn write_raw_buffer<B: RegisterBus>(
        &self,
        bus: &mut B,
        val: &[u8],
    ) -> Result<(), ErrorReg<B::Error>> {
        if !matches!(self.access, Mode::RW | Mode::W) {
            return Err(ErrorReg::ReadOnly);
        }
        bus.write_register(self.addr, val)?;
        Ok(())
    }

    // —————————————————————————————————————————— Other ————————————————————————————————————————————

    pub fn get_field_mask(&self, field_id: ENUM) -> Result<u32, ErrorReg<()>> {
        let fields = self.fields.ok_or(ErrorReg::NoFields)?;

        let field = fields
            .iter()
            .find(|f| f.id == field_id)
            .ok_or(ErrorReg::FieldNotFound)?;

        Ok(create_mask(field.width, field.offset))
    }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Functions
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[inline]
fn create_mask(width: u8, offset: u8) -> u32 {
    if width == 32 { u32::MAX } else { ((1u32 << width) - 1) << offset }
}

/// Static validation for the Registry definition at creation
pub const fn validated<ENUM: 'static + PartialEq>(reg: Register<ENUM>) -> Register<ENUM> {
    if reg.bytes == 0 || reg.bytes > MAX_BYTES {
        panic!("Register size must be 1 to 4 bytes");
    }

    let bits = (reg.bytes as u16) * 8;

    if let Some(list) = reg.fields {
        let mut i = 0;
        while i < list.len() {
            let f = &list[i]; // <-- use reference

            if f.width == 0 {
                panic!("Field width cannot be 0");
            }

            if (f.offset as u16) + (f.width as u16) > bits {
                panic!("Field exceeds register size");
            }

            // Check for field overlaps
            let mut j = 0;
            while j < list.len() {
                if i != j {
                    let other = &list[j];
                    let f_start = f.offset;
                    let f_end = f.offset + f.width - 1;
                    let o_start = other.offset;
                    let o_end = other.offset + other.width - 1;

                    if !(f_end < o_start || f_start > o_end) {
                        panic!("Fields overlap");
                    }
                }
                j += 1;
            }

            i += 1;
        }
    }

    reg
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Error
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorReg<BUS_ER: Debug> {
    ValueTooLarge,
    BufferOverrun,
    NoFields,
    ReadOnly,
    WriteOnly,
    ReqRW,
    WrongFieldDef,
    FieldNotFound,
    Bus(BUS_ER),
}

impl<BUS_ER: Debug> Display for ErrorReg<BUS_ER> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match &self {
            ErrorReg::ValueTooLarge => write!(f, "field value too large"),
            ErrorReg::BufferOverrun => write!(f, "buffer overrun"),
            ErrorReg::NoFields => write!(f, "register has no fields"),
            ErrorReg::ReadOnly => write!(f, "read only register"),
            ErrorReg::WriteOnly => write!(f, "write only register"),
            ErrorReg::ReqRW => write!(f, "requires read write access"),
            ErrorReg::WrongFieldDef => write!(f, "wrong field definition"),
            ErrorReg::FieldNotFound => write!(f, "field not found"),
            ErrorReg::Bus(e) => Debug::fmt(e, f),
        }
    }
}

impl<BUS_ER: Debug> From<BUS_ER> for ErrorReg<BUS_ER> {
    fn from(error: BUS_ER) -> Self {
        ErrorReg::Bus(error)
    }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Transport-agnostic bus trait
/// This trait needs to be implemented for the SPI, I2C bus used
pub trait RegisterBus {
    type Error: Debug;
    fn read_register(&mut self, reg_addr: u8, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write_register(&mut self, reg_addr: u8, buf: &[u8]) -> Result<(), Self::Error>;
}
