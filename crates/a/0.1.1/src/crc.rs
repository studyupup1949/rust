/// HJ 212—2025 "ANSI CRC16" (poly 0xA001, init 0xFFFF) as described in the standard text.
///
/// This matches the provided pseudocode shape:
/// `crc = (crc >> 8) ^ byte;` then 8 iterations of shift/xor with 0xA001.
pub fn crc16_ansi(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in bytes {
        crc = (crc >> 8) ^ (b as u16);
        for _ in 0..8 {
            let check = crc & 0x0001;
            crc >>= 1;
            if check == 0x0001 {
                crc ^= 0xA001;
            }
        }
    }
    crc
}

/// CRC16/Modbus (poly 0xA001, init 0xFFFF), used by many historical HJ212 implementations.
pub fn crc16_modbus(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in bytes {
        crc ^= b as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// Default CRC hex rendering used by frame builders/parsers: HJ212 ANSI CRC16.
pub fn crc16_hex_lower(bytes: &[u8]) -> String {
    format!("{:04x}", crc16_ansi(bytes))
}

/// Default CRC hex rendering used by frame builders/parsers: HJ212 ANSI CRC16.
pub fn crc16_hex_upper(bytes: &[u8]) -> String {
    format!("{:04X}", crc16_ansi(bytes))
}

/// Explicit CRC16/Modbus hex rendering (compat).
pub fn crc16_modbus_hex_lower(bytes: &[u8]) -> String {
    format!("{:04x}", crc16_modbus(bytes))
}

/// Explicit CRC16/Modbus hex rendering (compat).
pub fn crc16_modbus_hex_upper(bytes: &[u8]) -> String {
    format!("{:04X}", crc16_modbus(bytes))
}

pub fn is_hex4(s: &str) -> bool {
    s.len() == 4 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_ansi_matches_standard_example() {
        // Example from the standard text (as provided in the issue description)
        let payload = "QN=20240601085857223;ST=32;CN=1011;PW=123456;MN=010000A8900016F000169DC0;Flag=9;CP=&&&&";
        assert_eq!(payload.as_bytes().len(), 87);
        assert_eq!(crc16_hex_upper(payload.as_bytes()), "2200");
        // Make sure Modbus variant differs (to prevent accidental conflation).
        assert_eq!(crc16_modbus_hex_upper(payload.as_bytes()), "CCF4");
    }
}
