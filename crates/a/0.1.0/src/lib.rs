//! `a` implements the **HJ 212** ASCII protocol (protocol-layer only).
//!
//! This crate focuses on:
//! - **Receiving**: stream deframing (`Framer`) + frame parsing (`parse_frame`)
//! - **Sending**: building `CP=&&...&&` and payloads (`CpBuilder`, `PayloadBuilder`) + frame building (`build_frame`)
//!
//! It intentionally does **not** include networking/serial I/O stacks.
//!
//! # Parse a frame
//!
//! ```
//! use a::{build_frame, parse_frame};
//!
//! let payload = "QN=1;ST=22;CN=2011;PW=123456;MN=ABC;Flag=7;CP=&&DataTime=20250101010101;a21026-Rtd=12.3&&";
//! let frame = build_frame(payload);
//! let pkt = parse_frame(&frame).unwrap();
//!
//! assert_eq!(pkt.mn.as_deref(), Some("ABC"));
//! assert_eq!(pkt.cp.get("a21026-Rtd").map(String::as_str), Some("12.3"));
//! ```
//!
//! # Deframe from a stream (TCP/serial)
//!
//! ```no_run
//! use a::{Framer, parse_frame};
//!
//! let mut framer = Framer::new();
//! framer.push(b"##0025QN=1;ST=22;CN=2011;CP=&&&&");
//!
//! while let Some(frame_bytes) = framer.next_frame() {
//!     let frame = String::from_utf8(frame_bytes).expect("HJ212 is ASCII");
//!     let _pkt = parse_frame(&frame)?;
//! }
//!
//! # Ok::<(), a::Hj212Error>(())
//! ```
//!
//! # Build a sending frame
//!
//! ```
//! use a::{CpBuilder, PayloadBuilder, parse_frame};
//!
//! let mut cp = CpBuilder::new();
//! cp.data_time("20250101010101")
//!   .rtd_flag("a21026", "12.3", "N");
//!
//! let frame = PayloadBuilder::new("QN1", "123456", "ABC", cp.build()).frame();
//! let pkt = parse_frame(&frame).unwrap();
//! assert_eq!(pkt.mn.as_deref(), Some("ABC"));
//! ```

mod builder;
mod crc;
mod datatime;
mod error;
pub mod crypto;
mod flag;
mod frame;
mod framer;
mod packet;
mod multimedia;

pub use builder::{build_data_ack, build_exe_rtn, build_notify_ack, build_qn_rtn, CpBuilder, PayloadBuilder};
pub use crc::{
    crc16_ansi, crc16_hex_lower, crc16_hex_upper, crc16_modbus, crc16_modbus_hex_lower, crc16_modbus_hex_upper,
};
pub use crypto::{decrypt_data_segment, encrypt_data_segment, BlockCipher16, EncryptedDataSegment};
pub use datatime::parse_datatime_to_utc;
pub use error::Hj212Error;
pub use flag::Hj212Flag;
pub use frame::{build_frame, build_frame_compat, build_frame_standard, parse_frame, parse_frame_strict};
pub use framer::Framer;
pub use multimedia::{MediaFormat, MediaType};
pub use packet::Hj212Packet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_parse_roundtrip() {
        let payload = "QN=1;ST=22;CN=2011;PW=123;MN=ABC;Flag=7;CP=&&DataTime=20250101010101;a21026-Rtd=12.3&&";
        let frame = build_frame(payload);
        assert!(frame.ends_with("\r\n"));
        let pkt = parse_frame(&frame).unwrap();
        assert_eq!(pkt.mn.as_deref(), Some("ABC"));
        assert_eq!(pkt.data_time.as_deref(), Some("20250101010101"));
        assert_eq!(pkt.cp.get("a21026-Rtd").map(String::as_str), Some("12.3"));
    }

    #[test]
    fn framer_extracts_frames() {
        let payload = "QN=1;ST=22;CN=2011;PW=123;MN=ABC;Flag=7;CP=&&DataTime=20250101010101&&";
        let frame = build_frame(payload);
        let mut fr = Framer::new();
        fr.push(frame[..5].as_bytes());
        assert!(fr.next_frame().is_none());
        fr.push(frame[5..].as_bytes());
        let out = fr.next_frame().unwrap();
        assert_eq!(String::from_utf8_lossy(&out), frame);
    }

    #[test]
    fn standard_frame_roundtrip_and_suffix() {
        let payload = "QN=20250101010101001;ST=22;CN=2011;PW=123;MN=ABC;Flag=9;CP=&&DataTime=20250101010101&&";
        let frame = build_frame_standard(payload);
        assert!(frame.ends_with("\r\n"));
        let pkt = parse_frame_strict(&frame).unwrap();
        let flag = pkt.flag_bits().unwrap();
        assert_eq!(flag.version(), 2);
        assert_eq!(flag.need_ack(), true);
        assert_eq!(flag.has_packet(), false);
    }

    #[test]
    fn strict_parse_standard_text_example_frame() {
        // Standard text example (line breaks in documents are for formatting; payload bytes do not include '\n').
        let frame = "##0087QN=20240601085857223;ST=32;CN=1011;PW=123456;MN=010000A8900016F000169DC0;Flag=9;CP=&&&&2200\r\n";
        let pkt = parse_frame_strict(frame).unwrap();

        assert_eq!(pkt.length_hint, Some(87));
        assert_eq!(pkt.crc_hex.as_deref(), Some("2200"));

        assert_eq!(pkt.qn.as_deref(), Some("20240601085857223"));
        assert_eq!(pkt.st.as_deref(), Some("32"));
        assert_eq!(pkt.cn.as_deref(), Some("1011"));
        assert_eq!(pkt.pw.as_deref(), Some("123456"));
        assert_eq!(pkt.mn.as_deref(), Some("010000A8900016F000169DC0"));
        assert_eq!(pkt.flag.as_deref(), Some("9"));

        // Empty CP body: CP=&&&&
        assert!(pkt.cp.is_empty());
    }

    #[test]
    fn parse_user_provided_compat_frame_len453_crc_c0c1() {
        // User-provided (compat) frame: 3-digit LEN, lowercase CRC, no CRLF.
        let frame = "##453QN=20160801085000001;ST=22;CN=2011;PW=123456;MN=010000A8900016F000169DC0;Flag=7;CP=&&DataTime=20160801084000;a21005-Rtd=1.1,a21005-Flag=N;a21004-Rtd=112,a21004-Flag=N;a21026-Rtd=58,a21026-Flag=N;LA-td=50.1,LA-Flag=N;a34004-Rtd=207,a34004-Flag=N;a34002-Rtd=295,a34002-Flag=N;a01001-Rtd=12.6,a01001-Flag=N;a01002-Rtd=32,a01002-Flag=N;a01006-Rtd=101.02,a01006-Flag=N;a01007-Rtd=2.1,a01007-Flag=N;a01008-Rtd=120,a01008-Flag=N;a34001-Rtd=217,a34001-Flag=N;&&c0c1";

        // Validate LEN/CRC consistency under the crate's default ANSI CRC16.
        let bytes = frame.as_bytes();
        let mut idx = 2;
        while idx < bytes.len() && idx < 2 + 4 && (bytes[idx] as char).is_ascii_digit() {
            idx += 1;
        }
        let len_str = std::str::from_utf8(&bytes[2..idx]).unwrap();
        let declared_len: usize = len_str.parse().unwrap();
        assert_eq!(declared_len, 453);
        let payload_start = idx;
        let payload_end = payload_start + declared_len;
        let crc_end = payload_end + 4;
        assert!(bytes.len() >= crc_end);
        let payload = &bytes[payload_start..payload_end];
        let got_crc = std::str::from_utf8(&bytes[payload_end..crc_end]).unwrap();
        let expected_crc = crc16_hex_lower(payload);

        assert_eq!(got_crc.to_lowercase(), "c0c1");
        assert_eq!(expected_crc, "c0c1");

        // And it should parse successfully.
        let pkt = parse_frame(frame).unwrap();
        assert_eq!(pkt.length_hint, Some(453));
        assert_eq!(pkt.crc_hex.as_deref(), Some("c0c1"));
        assert_eq!(pkt.qn.as_deref(), Some("20160801085000001"));
        assert_eq!(pkt.mn.as_deref(), Some("010000A8900016F000169DC0"));
        assert_eq!(pkt.data_time.as_deref(), Some("20160801084000"));

        // Note: CP parsing splits only on ';', so commas remain inside values.
        assert_eq!(
            pkt.cp.get("a21005-Rtd").map(String::as_str),
            Some("1.1,a21005-Flag=N")
        );
    }
}
