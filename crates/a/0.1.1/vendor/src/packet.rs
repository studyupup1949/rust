use std::collections::BTreeMap;

use crate::flag::Hj212Flag;

#[derive(Debug, Clone)]
pub struct Hj212Packet {
    pub length_hint: Option<usize>,
    pub payload: String,
    pub crc_hex: Option<String>,

    pub qn: Option<String>,
    pub st: Option<String>,
    pub cn: Option<String>,
    pub pw: Option<String>,
    pub mn: Option<String>,
    pub flag: Option<String>,

    pub cp: BTreeMap<String, String>,
    pub data_time: Option<String>,
}

impl Hj212Packet {
    fn header_kv(&self, key: &str) -> Option<&str> {
        let (head, _) = self.payload.split_once("CP=&&")?;
        for part in head.split(';').filter(|p| !p.trim().is_empty()) {
            let part = part.trim();
            if let Some((k, v)) = part.split_once('=') {
                if k == key {
                    return Some(v.trim());
                }
            }
        }
        None
    }

    /// Parse `Flag` into bits per HJ 212—2025.
    pub fn flag_bits(&self) -> Option<Hj212Flag> {
        let raw = self.header_kv("Flag")?;
        let v = raw.parse::<u16>().ok()?;
        if v > u8::MAX as u16 {
            return None;
        }
        Some(Hj212Flag::new(v as u8))
    }

    /// Total packet count (`PNUM`) if present.
    pub fn pnum(&self) -> Option<u32> {
        self.header_kv("PNUM")?.parse::<u32>().ok()
    }

    /// Packet sequence number (`PNO`) if present.
    pub fn pno(&self) -> Option<u32> {
        self.header_kv("PNO")?.parse::<u32>().ok()
    }

    /// Whether this is a retransmitted frame (`RF=1`).
    pub fn is_retransmit(&self) -> bool {
        matches!(self.header_kv("RF"), Some("1"))
    }
}
