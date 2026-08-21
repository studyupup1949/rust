//! Content-Length typed header.
//!
//! See [`ContentLength`] docs.

use std::{convert::Infallible, str};

use actix_web::{
    error::ParseError,
    http::header::{
        from_one_raw_str, Header, HeaderName, HeaderValue, TryIntoHeaderValue, CONTENT_LENGTH,
    },
    HttpMessage,
};

/// `Content-Length` header, defined in [RFC 9110 §8.6].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Forwarded {}

impl ContentLength {
    /// Returns Content-Length value.
    pub fn into_inner(&self) -> usize {
        self.0
    }
}

impl str::FromStr for ContentLength {
    type Err = <usize as str::FromStr>::Err;

    #[inline]
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        let mut host = None;
        let mut scheme = None;
        let mut realip_remote_addr = None;

        for (name, val) in val
            // "for=1.2.3.4, for=5.6.7.8; scheme=https"
            .flat_map(|val| val.split(';'))
            // ["for=1.2.3.4, for=5.6.7.8", " scheme=https"]
            .flat_map(|vals| vals.split(','))
            // ["for=1.2.3.4", " for=5.6.7.8", " scheme=https"]
            .flat_map(|pair| {
                let mut items = pair.trim().splitn(2, '=');
                Some((items.next()?, items.next()?))
            })
        {
            // [(name , val      ), ...                                    ]
            // [("for", "1.2.3.4"), ("for", "5.6.7.8"), ("scheme", "https")]

            // taking the first value for each property is correct because spec states that first
            // "for" value is client and rest are proxies; multiple values other properties have
            // no defined semantics
            //
            // > In a chain of proxy servers where this is fully utilized, the first
            // > "for" parameter will disclose the client where the request was first
            // > made, followed by any subsequent proxy identifiers.
            // --- https://datatracker.ietf.org/doc/html/rfc7239#section-5.2

            match name.trim().to_lowercase().as_str() {
                "for" => realip_remote_addr.get_or_insert_with(|| unquote(val)),
                "proto" => scheme.get_or_insert_with(|| unquote(val)),
                "host" => host.get_or_insert_with(|| unquote(val)),
                "by" => {
                    // TODO: implement https://datatracker.ietf.org/doc/html/rfc7239#section-5.1
                    continue;
                }
                _ => continue,
            };
        }

        let scheme = scheme
            .or_else(|| first_header_value(req, &X_FORWARDED_PROTO))
            .or_else(|| req.uri.scheme().map(Scheme::as_str))
            .or_else(|| Some("https").filter(|_| cfg.secure()))
            .unwrap_or("http")
            .to_owned();

        let host = host
            .or_else(|| first_header_value(req, &X_FORWARDED_HOST))
            .or_else(|| req.headers.get(&header::HOST)?.to_str().ok())
            .or_else(|| req.uri.authority().map(Authority::as_str))
            .unwrap_or_else(|| cfg.host())
            .to_owned();

        let realip_remote_addr = realip_remote_addr
            .or_else(|| first_header_value(req, &X_FORWARDED_FOR))
            .map(str::to_owned);

        let peer_addr = req.peer_addr.map(|addr| addr.ip().to_string());

        ConnectionInfo {
            host,
            scheme,
            peer_addr,
            realip_remote_addr,
        }
    }
}

impl TryIntoHeaderValue for ContentLength {
    type Error = Infallible;

    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        Ok(HeaderValue::from(self.0))
    }
}

impl Header for ContentLength {
    fn name() -> HeaderName {
        CONTENT_LENGTH
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        let val = from_one_raw_str(msg.headers().get(Self::name()))?;

        // decoder prevents multiple CL headers
        debug_assert_eq!(msg.headers().get_all(Self::name()).count(), 1);

        Ok(val)
    }
}

impl From<ContentLength> for usize {
    fn from(ContentLength(len): ContentLength) -> Self {
        len
    }
}

impl From<usize> for ContentLength {
    fn from(len: usize) -> Self {
        ContentLength(len)
    }
}

impl PartialEq<usize> for ContentLength {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ContentLength> for usize {
    fn eq(&self, other: &ContentLength) -> bool {
        *self == other.0
    }
}

impl PartialOrd<usize> for ContentLength {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<ContentLength> for usize {
    fn partial_cmp(&self, other: &ContentLength) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::{assert_parse_eq, assert_parse_fail};

    #[test]
    fn missing_header() {
        assert_parse_fail::<ContentLength, _, _>([""; 0]);
        assert_parse_fail::<ContentLength, _, _>([""]);
    }

    #[test]
    fn bad_header() {
        assert_parse_fail::<ContentLength, _, _>(["-123"]);
        assert_parse_fail::<ContentLength, _, _>(["123_456"]);
        assert_parse_fail::<ContentLength, _, _>(["123.456"]);

        // too large for u64 (2^64, 2^64 + 1)
        assert_parse_fail::<ContentLength, _, _>(["18446744073709551616"]);
        assert_parse_fail::<ContentLength, _, _>(["18446744073709551617"]);

        // hex notation
        assert_parse_fail::<ContentLength, _, _>(["0x123"]);

        // multi-value
        assert_parse_fail::<ContentLength, _, _>(["0, 123"]);
    }

    #[test]
    #[should_panic]
    fn bad_header_plus() {
        // prevented by decoder anyway
        assert_parse_fail::<ContentLength, _, _>(["+123"]);
    }

    #[test]
    #[should_panic]
    fn bad_multiple_value() {
        assert_parse_fail::<ContentLength, _, _>(["0", "123"]);
    }

    #[test]
    fn good_header() {
        assert_parse_eq::<ContentLength, _, _>(["0"], ContentLength(0));
        assert_parse_eq::<ContentLength, _, _>(["1"], ContentLength(1));
        assert_parse_eq::<ContentLength, _, _>(["123"], ContentLength(123));

        // value that looks like octal notation is not interpreted as such
        assert_parse_eq::<ContentLength, _, _>(["0123"], ContentLength(123));

        // whitespace variations
        assert_parse_eq::<ContentLength, _, _>([" 0"], ContentLength(0));
        assert_parse_eq::<ContentLength, _, _>(["0 "], ContentLength(0));
        assert_parse_eq::<ContentLength, _, _>([" 0 "], ContentLength(0));

        // large value (2^64 - 1)
        assert_parse_eq::<ContentLength, _, _>(
            ["18446744073709551615"],
            ContentLength(18_446_744_073_709_551_615),
        );
    }

    #[test]
    fn equality() {
        assert!(ContentLength(0) == ContentLength(0));
        assert!(ContentLength(0) == 0);
        assert!(0 != ContentLength(123));
    }

    #[test]
    fn ordering() {
        assert!(ContentLength(0) < ContentLength(123));
        assert!(ContentLength(0) < 123);
        assert!(0 < ContentLength(123));
    }
}
