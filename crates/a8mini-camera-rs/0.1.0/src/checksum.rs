use crate::constants;

/// Computes CRC16 checksum given byte array. Output is little endian.
/// - G(X) = X^16+X^12+X^5+1
pub fn crc16_calc(arr: &[u8], crc_init: u16) -> [u8; 2] {
	let mut crc16: u16 = crc_init;
	
	for &val in arr {
		let temp = (crc16 >> 8) as u8;
		let oldcrc16 = constants::CRC16_TAB[(val ^ temp) as usize];
		crc16 = (crc16 << 8) ^ oldcrc16;
	}
	
	crc16.to_le_bytes()
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_crc16_hardcoded_checksums() {
		// GROUND TRUTH: https://crccalc.com/?crc=&method=CRC-16/XMODEM&datatype=hex&outtype=hex
		
		for cmd in constants::HARDCODED_COMMANDS {
			let computed_crc16 = crc16_calc(&cmd[.. (cmd.len() - 2)], 0);
			let expected_crc16 = &cmd[(cmd.len() - 2)..];
			assert_eq!(
				format!("{:x?}", computed_crc16),
				format!("{:x?}", expected_crc16),
			);
		}
	}
}