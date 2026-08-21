// lib.rs
// Aldaron's Codec Interface - PPM
// Copyright 2017 (c) Aldaron's Tech
// Copyright 2017 (c) Jeron Lau
// Licensed under the MIT LICENSE

#![no_std]

/// The errors that can be returned if `decode()` fails.
pub enum DecodeErrorPPM {
	/// Not a PPM file (bad header)
	NotPPM,
	/// Dimensions are not numbers
	BadNum,
}

impl ::core::fmt::Debug for DecodeErrorPPM {
	fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
		write!(f, "Couldn't parse PPM because: {}", match *self {
			DecodeErrorPPM::NotPPM => "Not a PPM file (bad header)",
			DecodeErrorPPM::BadNum => "Dimensions are not numbers",
		})
	}
}

fn skip_line(ppm: &'static [u8], index: &mut usize) {
	while ppm[*index] != b'\n' {
		*index += 1;
	}
	*index += 1;
}

fn utf8_to_u32(ppm: &'static [u8], index: &mut usize, until: u8)
	-> Result<u32, DecodeErrorPPM>
{
	let zero = b'0';
	let mut number = 0;

	while ppm[*index] != until {
		let digit = ppm[*index];

		number *= 10;
		if digit != zero {
			if digit < zero || digit > zero + 9 {
				return Err(DecodeErrorPPM::BadNum);
			}
			number += (digit - zero) as u32;
		}
		*index += 1;
	}
	*index += 1;
	Ok(number)
}

/// Decode PPM data.  On success, returns image as tuple:
/// `(width, height, pixels)`
pub fn decode(ppm: &'static [u8])
	-> Result<(u32, u32, &'static [u8]), DecodeErrorPPM>
{
	let mut index = 3;

	// Header
	if ppm[0] != b'P' || ppm[1] != b'6' {
		return Err(DecodeErrorPPM::NotPPM);
	}

	// Optional Comment
	if ppm[index] == b'#' {
		skip_line(ppm, &mut index);
	}

	// Width & Height
	let width = utf8_to_u32(ppm, &mut index, b' ')?;
	let height = utf8_to_u32(ppm, &mut index, b'\n')?;

	// We don't care about this.  In ppm format 255 is normally here
	skip_line(ppm, &mut index);

	Ok((width, height, &ppm[index..]))
}
