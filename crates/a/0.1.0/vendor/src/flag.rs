/// Parsed representation of the `Flag` field in HJ212.
///
/// Per HJ 212—2025 (6.3.3), `Flag` is an integer in range 0..=255, and bits map to:
/// - Bits 2..=7: standard version (V0..V5)
/// - Bit 1: whether packet fields (`PNUM`/`PNO`) exist (A)
/// - Bit 0: whether the command requires acknowledgement (D)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hj212Flag {
    raw: u8,
}

impl Hj212Flag {
    pub fn new(raw: u8) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u8 {
        self.raw
    }

    /// Standard version code (0..=63).
    ///
    /// Common values:
    /// - 0: HJ/T 212—2005
    /// - 1: HJ 212—2017
    /// - 2: HJ 212—2025
    pub fn version(self) -> u8 {
        self.raw >> 2
    }

    /// Whether `PNUM`/`PNO` fields are present.
    pub fn has_packet(self) -> bool {
        (self.raw & 0b0000_0010) != 0
    }

    /// Whether this command requires acknowledgement.
    pub fn need_ack(self) -> bool {
        (self.raw & 0b0000_0001) != 0
    }
}
