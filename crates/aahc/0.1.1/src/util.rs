pub(crate) mod io;

/// Checks whether a byte is a `tchar` (token character).
pub(crate) fn is_tchar(b: u8) -> bool {
	b.is_ascii_alphanumeric()
		|| b == b'!'
		|| b == b'#'
		|| b == b'$'
		|| b == b'%'
		|| b == b'&'
		|| b == b'\''
		|| b == b'*'
		|| b == b'+'
		|| b == b'-'
		|| b == b'.'
		|| b == b'^'
		|| b == b'_'
		|| b == b'`'
		|| b == b'|'
		|| b == b'~'
}

/// Checks whether a string is a token.
pub(crate) fn is_token(name: &str) -> bool {
	!name.is_empty() && name.bytes().all(is_tchar)
}

/// Checks whether a byte can legally appear in an HTTP header value.
pub(crate) fn is_field_vchar(b: u8) -> bool {
	b == b'\t' || b >= 0x20
}

/// Checks whether a sequence of bytes is a valid HTTP header value.
pub(crate) fn is_field_value(value: &[u8]) -> bool {
	if value.is_empty() {
		true
	} else {
		let first: u8 = *value.first().unwrap();
		let last: u8 = *value.last().unwrap();
		first != b' '
			&& first != b'\t'
			&& last != b' '
			&& last != b'\t'
			&& value.iter().all(|b| is_field_vchar(*b))
	}
}

/// Checks whether a byte is a valid character to appear in a `request-target`.
///
/// This check is relaxed and does not check the full requirements for path validity.
pub(crate) fn is_request_target_char(b: u8) -> bool {
	(0x21_u8..=0x7F_u8).contains(&b)
}

/// Checks whether a string is a `request-target`.
///
/// This check is relaxed and does not check the full requirements for path validity.
pub(crate) fn is_request_target(req: &str) -> bool {
	!req.is_empty() && req.bytes().all(is_request_target_char)
}

/// Scan the headers and determine whether the `connection` header is present and contains the
/// `close` option.
pub(crate) fn is_connection_close(headers: &[crate::Header<'_>]) -> bool {
	for header in headers {
		if header.name.eq_ignore_ascii_case("connection") {
			for mut option in header.value.split(|b| *b == b',') {
				while option.first() == Some(&b' ') || option.first() == Some(&b'\t') {
					option = &option[1..];
				}
				while option.last() == Some(&b' ') || option.last() == Some(&b'\t') {
					option = &option[..option.len() - 1];
				}
				if option.eq_ignore_ascii_case(b"close") {
					return true;
				}
			}
		}
	}
	false
}
