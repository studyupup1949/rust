// "aci_ppm" - Aldaron's Codec Interface / PPM
//
// Copyright Jeron A. Lau 2017-2018.
// Distributed under the Boost Software License, Version 1.0.  (See accompanying
// file LICENSE or copy at https://www.boost.org/LICENSE_1_0.txt)
//
//! Aldaron's Codec Interface / PPM is a small library developed by Plop Grizzly
//! to encode and decode ppm/pnm image files.
//
// # MMD (Multi-Media Deflated)
// MMD is a compressed (DEFLATE) variant of PNM, developed by Plop Grizzly.
//
// ## Initial Chunk Options ---
// 
// ### "5" - Image - 1 byte per channel RGB or all channels off
// * Little endian unsigned 16-bit width.
// * Little endian unsigned 16-bit height.
// * Repeating [0 or 1RGB] sequences.
//
// ### "4" - Image - 1 byte per channel RGBA
// * Little endian unsigned 16-bit width.
// * Little endian unsigned 16-bit height.
// * Repeating [RGBA] 4 byte sequences.
//
// ### "3" - Image - 1 byte per channel RGB
// * Little endian unsigned 16-bit width.
// * Little endian unsigned 16-bit height.
// * Repeating [RGB] 3 byte sequences.
//
// ### "2" - Image - 1 bit per pixel - Black & White / On & Off.
// * Little endian unsigned 16-bit width.
// * Little endian unsigned 16-bit height.
// * Repeating bit per pixel (B&W).
//
// ### "1" - Image - 1 byte per pixel - Grayscale
// * Little endian unsigned 16-bit width.
// * Little endian unsigned 16-bit height.
// * Repeating byte per pixel (Grayscale).
//
// ## Additional Chunks
//
// ### "0" - Image - Size & Channels inherited from previous image.
// * Repeating byte sequences.
//
// ### "D" - Delay
// * Little endian unsigned 32-bit delay 48000=1fps, 1600=30fps, 0=restart vid.
//
// ### "A" - Audio
// * Little endian unsigned 32-bit delay 48000=1fps, 1600=30fps, 0=ERROR.
// * 16-bit Samples equal to 32-bit delay (48000hz).
//
// ### "t" - Text / Title(name)
// ### "c" - Creation Year / Optional Full Date
// ### "a" - Artist Name
// ### "w" - Artist Website
// ### "p" - Publisher
// ### "d" - Publish Date
// ### "s" - Publisher Website
// ### "l" - License: Copyright ⓒ All Rights Reserved Copyright Holder Name
// * Little endian unsigned 32-bit length.
// * Unicode

#![warn(missing_docs)]
#![doc(html_logo_url="https://plopgrizzly.com/images/plopgrizzly-splash.png",
   html_favicon_url="https://plopgrizzly.com/images/plopgrizzly-splash.png")]

extern crate afi;
extern crate byteorder;

use std::io::{ Write, Read, Seek, Cursor, SeekFrom };
use afi::*;

/// Encoder for PNM.
pub struct PnmEncoder {
	// Image size.
	wh: (u16,u16),
	// 3(false) or 4(true) channels.
	format: ColorChannels,
}

impl EncoderV for PnmEncoder {
	fn new(video: &Video) -> PnmEncoder {
		let wh = video.wh();
		let format = video.format();

		PnmEncoder {
			wh, format
		}
	}

	fn run(&mut self, frame: &VFrame) -> Vec<u8> {
		let mut out = Vec::new();
		out.write_fmt(format_args!("P6\n{}\n{}\n255\n", self.wh.0,
			self.wh.1)).unwrap();
		for i in 0..((self.wh.0 as usize) * (self.wh.1 as usize)) {
			let [r, g, b, _] = frame.sample_rgba(self.format, i);
			out.write_all(&[r, g, b]).unwrap();
		}
		out
	}

	fn end(self) -> Vec<u8> {
		vec![]
	}
}

/// Decoder for PPM/PNM.
pub struct PnmDecoder<T: Read + Seek> {
	#[allow(unused)] // TODO
	data: T,
	channels: ColorChannels,
	wh: (u16, u16),
	n_frames: u32,
}

impl<T> Decoder<T> for PnmDecoder<T> where T: Read + Seek {
	fn new(mut data: T, channels: afi::ColorChannels)
		-> Option<PnmDecoder<T>>
	{
		// Check Header: Is PNM (PPM, TODO: other PNM formats)
		let mut header = [0u8; 2];
		data.read(&mut header).unwrap();
		match &header {
			b"P6" => { /* P6 = PPM = RGB */ }
			_ => {
				/* File ain't be PNM */
				return None;
			}
		}

		// Skip Optional Comment
		let mut optional_comment = [0u8; 1];
		data.read(&mut optional_comment).unwrap();

		match &optional_comment {
			b"#" => { skip(&mut data); }
			_ => { data.seek(SeekFrom::Current(-1)).unwrap(); }
		}

		// Get Width & Height
		let width = utf8_to_u16(&mut data)?;
		let height = utf8_to_u16(&mut data)?;
		let max = utf8_to_u16(&mut data)?;
		let wh = (width, height);

		// Max pixel value should be 255.
		if max != 255 {
			println!("WARNING: PNM decoder requires 255 max");
			return None;
		}

		// TODO: Calculate chunks from remaining file size.
		let n_frames = 1;

		Some(PnmDecoder { data, channels, wh, n_frames })
	}

	fn run(&mut self, audio: &mut Option<Audio>, video: &mut Option<Video>)
		-> Option<bool>
	{
		if audio.is_none() && video.is_none() {
			*video = Some(Video::new(
				self.channels, self.wh, self.n_frames
			));

			// First run, initialize structs, some left.
			Some(true)
		} else {
			// Non-first run, decode 1 frame.
			let ch = self.channels.n_channels();
			let size = self.wh.0 as usize * self.wh.1 as usize * ch;
			let mut out: Vec<u8> = Vec::with_capacity(size);

			// Build Graphic
			for _ in 0..size {
				let mut rgb = [0u8; 3];
				self.data.read(&mut rgb).unwrap();

				let rgba = self.channels.from(Rgba,
					[rgb[0], rgb[1], rgb[2], 255u8]);

				for i in 0..ch {
					out.push(rgba[i]);
				}
			}

			let video = video.as_mut().unwrap(); // Isn't none, so fine.
			video.add(VFrame(out));

			// TODO, return True if APNM and has more frames.
			Some(false)
		}
	}

	fn get(&self) -> Index {
		// TODO
		Index(0)
	}

	fn set(&mut self, _index: Index) {
		// TODO
	}
}

/// Simple API for quickly loading entire files all at once.
pub fn decode(file: &[u8], channels: ColorChannels) -> Option<Video> {
	let mut image = None;
	let mut decoder = PnmDecoder::new(Cursor::new(file), channels)?;
	decoder.run(&mut None, &mut image).unwrap();
	decoder.run(&mut None, &mut image).unwrap();
	image
}

/// Skip until the next whitespace.
fn skip<T>(data: &mut T) where T: Read + Seek {
	loop {
		let mut character = [0u8; 1];
		data.read(&mut character).unwrap();
		match &character {
			b"\n" | b" " | b"\t" => break,
			_ => { /* continue */ }
		}
	}
}

/// Read number until whitespace.
fn utf8_to_u16<T>(data: &mut T) -> Option<u16> where T: Read + Seek {
	let mut number = 0;
	let zero = b'0';

	loop {
		let mut character = [0u8; 1];
		data.read(&mut character).unwrap();
		match &character {
			b"\n" | b" " | b"\t" => break,
			digit => {
				let digit = digit[0];
				number *= 10;
				if digit != zero {
					if digit < zero || digit > zero + 9 {
						return None;
					}
					number += (digit - zero) as u16;
				}
			}
		}
	}

	Some(number)
}

/*/// Encode an MMD Image
pub fn encode_mmd(mut graphic: Graphic, alpha: bool) -> Vec<u8> {
	// Convert to RGBA bytes.
	graphic.rgba();
	let graphic = graphic.as_bytes();

	// Build the encoded data.
	let mut enc = DeflateEncoder::new(Vec::new(), Compression::Best);

	if alpha {
		enc.write_all(b"4").unwrap();
	} else {
		enc.write_all(b"3").unwrap();
	}
	enc.write_u16::<byteorder::LittleEndian>(graphic.0).unwrap();
	enc.write_u16::<byteorder::LittleEndian>(graphic.1).unwrap();
	if alpha {
		enc.write_all(graphic.2).unwrap();
	} else {
		let size = graphic.0 as usize * graphic.1 as usize;

		for i in 0..size {
			enc.write_all(&[graphic.2[i * 4 + 0],
				graphic.2[i * 4 + 1], graphic.2[i * 4 + 2]]
			).unwrap();
		}
	}
	enc.finish().unwrap()
}

/// Decode an MMD Image
pub fn decode_mmd(ppm: &[u8]) -> Result<Graphic, ()> {
	// Decompress the data.
	let dec = inflate_bytes(ppm).unwrap();
	let mut cur = ::std::io::Cursor::new(dec);

	// Header - First is always image
	let mut header = [0; 1];
	cur.read(&mut header).unwrap();

	let alpha = match header {
		[b'4'] => true,
		[b'3'] => false,
		_ => return Err(())
	};

	// Width & Height
	let width = cur.read_u16::<byteorder::LittleEndian>().unwrap();
	let height = cur.read_u16::<byteorder::LittleEndian>().unwrap();

	// Pixels
	let size = width as usize * height as usize;
	let mut out: Vec<u32> = Vec::with_capacity(size);

	if alpha {
		for _ in 0..size {
			out.push(cur.read_u32::<byteorder::LittleEndian>().unwrap());
		}
	} else {
		let mut buf = [0; 3];
		for _ in 0..size {
			cur.read(&mut buf).unwrap();

			out.push(std::io::Cursor::new([buf[0], buf[1], buf[2],
					255u8])
				.read_u32::<byteorder::LittleEndian>()
				.unwrap()
			);
		}
	}

	// Graphic
	Ok(GraphicBuilder::new().rgba(width, height, out))
}*/
