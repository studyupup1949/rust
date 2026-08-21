//! A parser for Android boot image headers (e.g. `boot.img` or `vendor_boot.img`).
//!
//! This can be used to extract or patch e.g. the kernel or ramdisk.
//!
//! Byte array fields (`[u8; N]`) can be used as null-terminated strings.
//!
//! [`Header`] denotes the standard `boot.img` boot image's header with the file signature
//! `ANDROID!`. [`VendorHeader`] denotes the `vendor_boot.img` vendor boot image's header with the
//! file signature `VNDRBOOT`.
//!
//! # Examples
//!
//! ```no_run
//! use std::fs::File;
//! use abootimg_oxide::{binrw::io::BufReader, Header};
//!
//! let mut r = BufReader::new(File::open("boot_a.img").unwrap());
//! let hdr = Header::parse(&mut r).unwrap();
//! println!("{hdr:#?}");
//!
//! // Extract the kernel
//! use std::io::{self, BufWriter, Read, Seek, SeekFrom};
//!
//! let mut w = BufWriter::new(File::create("boot_a_kernel").unwrap());
//! let r = r.get_mut();
//! r.seek(SeekFrom::Start(hdr.kernel_position() as u64))
//!     .unwrap();
//! io::copy(&mut r.take(hdr.kernel_size().into()), w.get_mut()).unwrap();
//! ```
//!
//! # Features
#![doc = document_features::document_features!()]
//!
//! # Note about `Seek` requirement
//!
//! [`binrw`](mod@binrw) requires the [`Seek` trait](binrw::io::Seek) to be implemented on readers and writers.
//!
//! Only the read portion of [`Header`] seeks. For other functionality, you can use the
//! [`binrw::io::NoSeek`] adapter to be able to read and write to and from unseekable streams.

#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

/// Re-export for ease of use
pub use binrw;

/// **DO NOT USE**: This re-export will be removed at the next major release.
#[deprecated(
    since = "0.2.3",
    note = "Will be removed at the next major release. Use `abootimg_oxide::binrw::io::BufReader` instead."
)]
#[doc(no_inline)]
#[cfg(feature = "std")]
pub use binrw::io::BufReader;

mod standard;
mod vendor;
mod version;

/// Either variants of Android boot image header.
#[binrw::binrw]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[brw(little)]
pub enum EitherHeader {
    /// Standard Android boot image header, with file signature `ANDROID!`
    Standard(
        // TODO: ignores endian...
        #[br(parse_with = |r, _, ()| Header::parse(r))]
        #[bw(write_with = |hdr, w, _, ()| hdr.write(w))]
        Header,
    ),
    /// Android vendor boot image header, with file signature `VNDRBOOT`
    Vendor(VendorHeader),
}

pub use standard::{Header, HeaderV0, HeaderV0Versioned, HeaderV3};
pub use vendor::{VendorHeader, VendorHeaderV4};
pub use version::{OsPatch, OsVersion, OsVersionPatch};

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use binrw::{io::Cursor, BinRead};

    use super::*;

    #[track_caller]
    fn check<T: core::fmt::Debug, E: core::fmt::Display>(
        res: Result<T, E>,
        target_err_msgs: &[&str],
    ) {
        let s = res.unwrap_err().to_string();
        for target in target_err_msgs {
            assert!(
                s.contains(target),
                "---\\\n{s}\n\\--- should contain {target:?}"
            );
        }
    }

    #[test]
    fn invalid_file_signature_either() {
        let data = b"aaaaaaaa";
        check(
            EitherHeader::read(&mut Cursor::new(data)),
            &["no variants matched", "bad magic"],
        );
    }
    #[test]
    fn invalid_version_either() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ANDROID!");
        data.append(&mut b"aaaa".repeat(8));
        data.extend_from_slice(&u32::MAX.to_le_bytes());
        data.extend_from_slice(b"aaaa");
        data.append(&mut b"a".repeat(16 + 512 + 32 + 1024));

        check(
            HeaderV0::read(&mut Cursor::new(&data)),
            &["invalid header version"],
        );
    }
    #[test]
    fn invalid_version_direct_v0() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ANDROID!");
        data.append(&mut b"aaaa".repeat(8));
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(b"aaaa");
        data.append(&mut b"a".repeat(16 + 512 + 32 + 1024));

        check(
            HeaderV0::read(&mut Cursor::new(&data)),
            &["invalid header version"],
        );
    }
    #[test]
    fn invalid_version_direct_v3() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ANDROID!");
        data.append(&mut b"aaaa".repeat(4));
        data.append(&mut b"a".repeat(16));
        data.extend_from_slice(&0u32.to_le_bytes());
        data.append(&mut b"a".repeat(512 + 1024));

        check(
            HeaderV3::read(&mut Cursor::new(&data)),
            &["invalid header version"],
        );
    }

    #[test]
    fn invalid_size_direct_v3() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ANDROID!");
        data.append(&mut b"aaaa".repeat(4));
        data.append(&mut b"a".repeat(16));
        data.extend_from_slice(&3u32.to_le_bytes());
        data.append(&mut b"a".repeat(512 + 1024));

        check(
            HeaderV3::read(&mut Cursor::new(&data)),
            &["invalid header size"],
        );
    }

    #[test]
    fn invalid_size_either_v3() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ANDROID!");
        data.append(&mut b"aaaa".repeat(4));
        data.append(&mut b"a".repeat(16));
        data.extend_from_slice(&3u32.to_le_bytes());
        data.append(&mut b"a".repeat(512 + 1024));

        check(
            EitherHeader::read(&mut Cursor::new(&data)),
            &["invalid header size"],
        );
    }
}
