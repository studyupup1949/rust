use crate::ParseError;

/// Parses the `address` with the `parse` function, lowercasing the address first when it contains uppercase ASCII.
///
/// Borrowed parsers require lowercase input, so this normalizes owned-type parsing without allocating for
/// already-lowercase input.
pub(crate) fn parse_lowercase<T>(
    address: &str,
    parse: impl FnOnce(&str) -> Result<T, ParseError>,
) -> Result<T, ParseError> {
    if address.bytes().any(|b| b.is_ascii_uppercase()) {
        parse(address.to_ascii_lowercase().as_str())
    } else {
        parse(address)
    }
}
