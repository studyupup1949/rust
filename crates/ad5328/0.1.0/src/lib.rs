#![cfg_attr(not(test), no_std)]

use embedded_hal::{blocking::spi::Write, digital::v2::OutputPin};

#[repr(u8)]
/// All available DAC channels (A..H). These are configurable in two groups: A...D and E...H.
#[derive(Clone, Copy)]
pub enum Channel {
    /// DAC Channel A
    A,
    /// DAC Channel B
    B,
    /// DAC Channel C
    C,
    /// DAC Channel D
    D,
    /// DAC Channel E
    E,
    /// DAC Channel F
    F,
    /// DAC Channel G
    G,
    /// DAC Channel H
    H,
}

impl From<Channel> for u8 {
    fn from(chan: Channel) -> Self {
        chan as u8
    }
}

impl Channel {
    fn as_u16(&self) -> u16 {
        (*self as u16) << 12
    }
}

#[derive(Debug)]
pub enum Error<S, P> {
    /// SPI bus error
    Spi(S),
    /// Pin error
    Pin(P),
    /// Connection error (device not found)
    Conn,
    /// Address error (invalid or out of bounds)
    Address,
    /// Port error (invalid or out of bounds)
    Port,
    /// Out of bounds error
    Oob,
}

#[repr(u8)]
#[derive(Clone, Copy)]
/// The gain of the DACs is controlled by setting Bit 4 for the first group of DACs (A, B, C, and D) and Bit 5 for the second group of DACs (E, F, G, and H).
pub enum GAIN {
    /// Output range of 0V to Vref
    Gain0Vref,
    /// Output range of 0V to 2*Vref
    Gain02Vref,
}

impl GAIN {
    fn as_u16(&self) -> u16 {
        (*self as u16) << 4
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
/// This controls whether the reference of a group of DACs is buffered or unbuffered. The reference of the first group of DACs (A, B, C, and D) is controlled by setting Bit 2, and the second group of DACs (E, F, G, and H) is controlled by setting Bit 3.  
pub enum BUF {
    /// Unbuffered reference
    Unbuffered,
    /// Buffered reference
    Buffered,
}

impl BUF {
    fn as_u16(&self) -> u16 {
        (*self as u16) << 2
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
/// These bits are set when VDD is to be used as a reference. The first group of DACs (A, B, C, and D) can be set up to use VDD by setting Bit 0, and the second group of DACs (E, F, G, and H) by setting Bit 1. The VDD bits have priority over the BUF bits. When VDD is used as the reference, it is always unbuffered and has an output range of 0 V to VREF regardless of the state of the GAIN and BUF bits.
pub enum VDD {
    /// Use external voltage reference
    ExternalRef,
    /// Use VDD as voltage reference
    VddAsRef,
}

impl VDD {
    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
/// LDAC mode controls LDAC, which determines when data is transferred from the input registers to the DAC registers. There are three options when updating the DAC registers, as shown in Table 8 (DS p17). If the user wishes to update the DAC through software, the LDAC pin should be tied high and the LDAC mode bits set as required. Alternatively, if the user wishes to control the DAC through hardware, that is, the LDAC pin, the LDAC mode bits should be set to LDAC high (default mode).
pub enum LDAC {
    /// This option sets LDAC permanently low, VDD allowing the DAC registers to be updated continuously
    LdacLow,
    /// This option sets LDAC permanently high. The DAC registers are latched and the input registers can change without affecting the contents of the DAC registers. This is the default option for this mode
    LdacHigh,
    /// This option causes a single pulse on LDAC, updating the DAC registers once.
    LdacSingleUpdate,
}

impl LDAC {
    fn as_u16(&self) -> u16 {
        0xa000 | (*self as u16)
    }
}

/// Configures GAIN, BUF and VDD bits (for channels A...D and E...H respectively) as well as LDAC behavior (for all channels)
pub struct Ad5328Config {
    pub gain: (GAIN, GAIN),
    pub buf: (BUF, BUF),
    pub vdd: (VDD, VDD),
    pub ldac: LDAC,
}

impl Default for Ad5328Config {
    fn default() -> Self {
        Self {
            gain: (GAIN::Gain0Vref, GAIN::Gain0Vref),
            buf: (BUF::Buffered, BUF::Buffered),
            vdd: (VDD::ExternalRef, VDD::ExternalRef),
            ldac: LDAC::LdacHigh,
        }
    }
}

impl Ad5328Config {
    /// Serialize the config as two easily digestible commands
    fn as_commands(&self) -> [u16; 2] {
        [
            0x8000
                | self.gain.0.as_u16()
                | self.gain.1.as_u16() << 1
                | self.buf.0.as_u16()
                | self.buf.1.as_u16() << 1
                | self.vdd.0.as_u16()
                | self.vdd.1.as_u16() << 1,
            self.ldac.as_u16(),
        ]
    }
}

pub struct Ad5328<SPI, EN> {
    spi: SPI,
    enable: EN,
    cmd_buf: [u8; 2],
}

impl<SPI, EN, S, P> Ad5328<SPI, EN>
where
    SPI: Write<u8, Error = S>,
    EN: OutputPin<Error = P>,
{
    fn write(&mut self, cmd: u16) -> Result<(), Error<S, P>> {
        self.enable.set_low().map_err(Error::Pin)?;
        self.cmd_buf[0] = (cmd >> 8) as u8;
        self.cmd_buf[1] = (cmd & 0xff) as u8;
        self.spi.write(&self.cmd_buf).map_err(Error::Spi)?;
        self.enable.set_high().map_err(Error::Pin)?;
        Ok(())
    }

    /// Initialize a new Ad5328 instance, while configuring it for the first time
    /// # Arguments
    ///
    /// * `spi` - embedded-hal compatible SPI instance
    /// * `enable` - embedded-hal compatible GPIO pin
    /// * `config` - The Ad5328 device configuration struct
    ///
    /// # Example
    ///
    /// ```
    /// // Get `spi` and `enable` from your embedded-hal
    /// let config = Ad5328Config {
    ///     // to use Vdd as the voltage reference for all channels of the DAC
    ///     vdd: (VDD::VddAsRef, VDD::VddAsRef),
    ///     ..Default::default()
    /// };
    /// let dac = Ad5328::init(spi, enable, config).unwrap();
    /// ```
    pub fn init(spi: SPI, enable: EN, config: Ad5328Config) -> Result<Self, Error<S, P>> {
        let mut ad5328 = Self {
            spi,
            enable,
            cmd_buf: [0; 2],
        };
        ad5328.configure(config)?;
        Ok(ad5328)
    }

    /// (Re-)configure the already initialized Ad5328
    pub fn configure(&mut self, config: Ad5328Config) -> Result<(), Error<S, P>> {
        for cmd in config.as_commands() {
            self.write(cmd)?;
        }
        Ok(())
    }

    /// Reset all DAC data. A full reset will also reset all control data
    pub fn reset(&mut self, full_reset: bool) -> Result<(), Error<S, P>> {
        let cmd = if full_reset { 0xf000 } else { 0xe000 };
        self.write(cmd)?;
        Ok(())
    }

    /// Power down the channels that are set to true in their respective position
    /// Channel A -> 0, ..., Channel H -> 7
    pub fn power_down(&mut self, channels: [bool; 8]) -> Result<(), Error<S, P>> {
        let mut cmd = 0xc000;
        for (n, &power_down) in channels.iter().enumerate() {
            cmd |= (if power_down { 1 } else { 0 }) << n;
        }
        self.write(cmd)?;
        Ok(())
    }

    /// Set the value for a DAC channel. Max value is 4095
    pub fn set_channel(&mut self, channel: Channel, value: u16) -> Result<(), Error<S, P>> {
        if value > 4095 {
            return Err(Error::Oob);
        }
        let cmd = channel.as_u16() | value;
        self.write(cmd)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
