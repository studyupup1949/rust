use std::{
    borrow::Cow,
    io::{Error, ErrorKind},
};

/// Resolve common encoding labels to encoding_rs constants
fn resolve_encoding(label: &str) -> Option<&'static encoding_rs::Encoding> {
    match label {
        // Fast path for default cases
        "utf-8" | "UTF-8" => Some(encoding_rs::UTF_8),
        "windows-1252" | "cp1252" | "iso-8859-1" | "latin1" => Some(encoding_rs::WINDOWS_1252),

        // Common aliases
        "shift_jis" | "shift-jis" | "sjis" => Some(encoding_rs::SHIFT_JIS),
        "big5" => Some(encoding_rs::BIG5),
        "gbk" | "gb18030" => Some(encoding_rs::GBK),
        "euc-kr" | "euckr" => Some(encoding_rs::EUC_KR),
        "iso-2022-jp" => Some(encoding_rs::ISO_2022_JP),
        "windows-1251" => Some(encoding_rs::WINDOWS_1251),
        "windows-1250" => Some(encoding_rs::WINDOWS_1250),
        "iso-8859-2" => Some(encoding_rs::ISO_8859_2),
        "iso-8859-5" => Some(encoding_rs::ISO_8859_5),
        "iso-8859-6" => Some(encoding_rs::ISO_8859_6),
        "iso-8859-7" => Some(encoding_rs::ISO_8859_7),
        "iso-8859-8" => Some(encoding_rs::ISO_8859_8),
        "euc-jp" | "eucjp" => Some(encoding_rs::EUC_JP),

        // Fallback to dynamic lookup
        _ => encoding_rs::Encoding::for_label(label.as_bytes()),
    }
}

pub fn decode<'a>(
    data: &'a [u8],
    encoding: Option<&str>,
    encoder_errors: Option<&str>,
) -> Result<Cow<'a, str>, Error> {
    // Fast path: UTF-8 (most common case)
    if encoding
        .map(|e| e.eq_ignore_ascii_case("utf-8"))
        .unwrap_or(false)
        || encoding.is_none()
    {
        return match encoder_errors {
            Some("strict") => std::str::from_utf8(data)
                .map(Cow::Borrowed)
                .map_err(|e| Error::new(ErrorKind::InvalidData, format!("invalid utf-8: {e}"))),
            _ => Ok(String::from_utf8_lossy(data)),
        };
    }

    // Determine default encoding: windows-1252 on Windows, UTF-8 elsewhere
    let label = encoding.unwrap_or(if cfg!(windows) {
        "windows-1252"
    } else {
        "utf-8"
    });

    // Resolve encoding (supports common aliases)
    let enc = resolve_encoding(label)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("unknown encoding: {label}")))?;

    // Handle error modes
    match encoder_errors {
        Some("strict") => enc
            .decode_without_bom_handling_and_without_replacement(data)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "strict decoding failed: malformed input",
                )
            })
            .map(|cow| cow),
        Some("ignore") | Some("replace") | None => {
            let (cow, _had_errors) = enc.decode_without_bom_handling(data);
            Ok(cow)
        }
        Some(mode) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid error mode: {mode} (expected: strict, ignore, replace)"),
        )),
    }
}
