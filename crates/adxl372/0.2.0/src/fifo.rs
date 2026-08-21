//! FIFO decoding utilities.

use crate::error::Result;
use crate::interface::Adxl372Interface;
use crate::params::{FifoFormat, FifoMode};

/// A decoded FIFO sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// X-axis reading, if enabled.
    pub x: Option<i16>,
    /// Y-axis reading, if enabled.
    pub y: Option<i16>,
    /// Z-axis reading, if enabled.
    pub z: Option<i16>,
    /// Indicates whether this sample corresponds to a peak event.
    pub is_peak: bool,
}

/// Snapshot of the FIFO control configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FifoSettings {
    /// FIFO watermark level expressed in samples.
    pub watermark: u16,
    /// FIFO operating mode.
    pub mode: FifoMode,
    /// FIFO packing format.
    pub format: FifoFormat,
}

impl FifoSettings {
    /// Creates a new settings snapshot.
    pub const fn new(watermark: u16, mode: FifoMode, format: FifoFormat) -> Self {
        Self {
            watermark,
            mode,
            format,
        }
    }
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            z: None,
            is_peak: false,
        }
    }
}

/// Reads raw FIFO bytes into the caller-provided buffer.
pub fn read_fifo_raw<IFACE>(interface: &mut IFACE, buf: &mut [u8]) -> Result<usize, IFACE::Error>
where
    IFACE: Adxl372Interface,
{
    if buf.is_empty() {
        return Ok(0);
    }

    interface.read_many(crate::registers::REG_STATUS, buf)?;
    Ok(buf.len())
}

/// Decodes FIFO samples into the provided output slice.
pub fn read_fifo_samples<IFACE>(
    interface: &mut IFACE,
    samples: &mut [Sample],
) -> Result<usize, IFACE::Error>
where
    IFACE: Adxl372Interface,
{
    let mut raw = [0u8; 6];
    let mut count = 0;

    for sample in samples.iter_mut() {
        let bytes_used = read_fifo_raw(interface, &mut raw)?;
        if bytes_used < 2 {
            break;
        }

        sample.x = Some(i16::from_be_bytes([raw[0], raw[1]]));
        sample.y = Some(i16::from_be_bytes([raw[2], raw[3]]));
        sample.z = Some(i16::from_be_bytes([raw[4], raw[5]]));
        sample.is_peak = false;
        count += 1;
    }

    Ok(count)
}
