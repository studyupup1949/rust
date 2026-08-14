/// Appendix H (HJ 212—2025) multimedia file transfer parameters.
///
/// This module intentionally contains only **code mappings** (H.3/H.4).
/// HTTPS upload, multipart/form-data, retries, etc. belong in the application layer.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Text,
    Other,
}

impl MediaType {
    pub fn code(self) -> u8 {
        match self {
            MediaType::Image => 1,
            MediaType::Audio => 2,
            MediaType::Video => 3,
            MediaType::Text => 4,
            MediaType::Other => 9,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(MediaType::Image),
            2 => Some(MediaType::Audio),
            3 => Some(MediaType::Video),
            4 => Some(MediaType::Text),
            9 => Some(MediaType::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaFormat {
    Jpeg,
    Tif,
    Mp3,
    Wav,
    Mp4,
    Wmv,
    Json,
    Xml,
    Txt,
    Pdf,
    Other,
}

impl MediaFormat {
    pub fn code(self) -> u8 {
        match self {
            MediaFormat::Jpeg => 1,
            MediaFormat::Tif => 2,
            MediaFormat::Mp3 => 3,
            MediaFormat::Wav => 4,
            MediaFormat::Mp4 => 5,
            MediaFormat::Wmv => 6,
            MediaFormat::Json => 7,
            MediaFormat::Xml => 8,
            MediaFormat::Txt => 9,
            MediaFormat::Pdf => 10,
            MediaFormat::Other => 99,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(MediaFormat::Jpeg),
            2 => Some(MediaFormat::Tif),
            3 => Some(MediaFormat::Mp3),
            4 => Some(MediaFormat::Wav),
            5 => Some(MediaFormat::Mp4),
            6 => Some(MediaFormat::Wmv),
            7 => Some(MediaFormat::Json),
            8 => Some(MediaFormat::Xml),
            9 => Some(MediaFormat::Txt),
            10 => Some(MediaFormat::Pdf),
            99 => Some(MediaFormat::Other),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_type_codes_roundtrip() {
        for (ty, code) in [
            (MediaType::Image, 1),
            (MediaType::Audio, 2),
            (MediaType::Video, 3),
            (MediaType::Text, 4),
            (MediaType::Other, 9),
        ] {
            assert_eq!(ty.code(), code);
            assert_eq!(MediaType::from_code(code), Some(ty));
        }
        assert_eq!(MediaType::from_code(0), None);
    }

    #[test]
    fn media_format_codes_roundtrip() {
        for (fmt, code) in [
            (MediaFormat::Jpeg, 1),
            (MediaFormat::Tif, 2),
            (MediaFormat::Mp3, 3),
            (MediaFormat::Wav, 4),
            (MediaFormat::Mp4, 5),
            (MediaFormat::Wmv, 6),
            (MediaFormat::Json, 7),
            (MediaFormat::Xml, 8),
            (MediaFormat::Txt, 9),
            (MediaFormat::Pdf, 10),
            (MediaFormat::Other, 99),
        ] {
            assert_eq!(fmt.code(), code);
            assert_eq!(MediaFormat::from_code(code), Some(fmt));
        }
        assert_eq!(MediaFormat::from_code(0), None);
    }
}
