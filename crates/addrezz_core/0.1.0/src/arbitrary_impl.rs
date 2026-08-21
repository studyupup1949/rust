//! `arbitrary::Arbitrary` implementation for [`Addr`].
//!
//! Generates addresses by sampling a scheme, an authority shape (domain
//! or IP literal), an optional port, and optional path/query/fragment
//! segments. The output is always a valid `Addr` — it's constructed
//! directly rather than round-tripped through the parser.

use arbitrary::{Arbitrary, Result, Unstructured};
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{Addr, Host, Scheme, Userinfo};

const SCHEMES: &[Scheme] = &[
    Scheme::Http,
    Scheme::Https,
    Scheme::Ws,
    Scheme::Wss,
    Scheme::Ssh,
    Scheme::Sftp,
    Scheme::Ftp,
    Scheme::Postgres,
    Scheme::Redis,
    Scheme::Mongodb,
];

const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

impl<'a> Arbitrary<'a> for Addr {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let scheme = SCHEMES[u.int_in_range(0..=SCHEMES.len() - 1)?].clone();

        let host = match u.int_in_range(0..=2)? {
            0 => Host::Ipv4(Ipv4Addr::from(u.arbitrary::<[u8; 4]>()?)),
            1 => Host::Ipv6(Ipv6Addr::from(u.arbitrary::<[u8; 16]>()?)),
            _ => Host::Domain(arbitrary_domain(u)?),
        };

        let userinfo = if u.arbitrary::<bool>()? {
            Some(Userinfo {
                username: arbitrary_label(u)?,
                password: if u.arbitrary::<bool>()? {
                    Some(arbitrary_label(u)?)
                } else {
                    None
                },
            })
        } else {
            None
        };

        let port = if u.arbitrary::<bool>()? {
            Some(u.int_in_range::<u16>(1..=65535)?)
        } else {
            None
        };

        let path = if u.arbitrary::<bool>()? {
            format!("/{}", arbitrary_label(u)?)
        } else {
            "/".to_string()
        };

        let query = if u.arbitrary::<bool>()? {
            Some(format!("{}={}", arbitrary_label(u)?, arbitrary_label(u)?))
        } else {
            None
        };

        let fragment = if u.arbitrary::<bool>()? {
            Some(arbitrary_label(u)?)
        } else {
            None
        };

        Ok(Addr {
            scheme,
            userinfo,
            host,
            port,
            path,
            query,
            fragment,
        })
    }
}

/// Letter-leading ASCII label. Ensures `url::Url` accepts the resulting
/// domain (it rejects labels that start with a digit because they
/// ambiguate with IPv4 literals).
fn arbitrary_label(u: &mut Unstructured<'_>) -> Result<String> {
    let len = u.int_in_range::<usize>(1..=12)?;
    let mut s = String::with_capacity(len);
    let first = u.int_in_range(0..=ALPHA.len() - 1)?;
    s.push(ALPHA[first] as char);
    for _ in 1..len {
        let idx = u.int_in_range(0..=ALNUM.len() - 1)?;
        s.push(ALNUM[idx] as char);
    }
    Ok(s)
}

fn arbitrary_domain(u: &mut Unstructured<'_>) -> Result<String> {
    let labels = u.int_in_range::<usize>(2..=4)?;
    let mut parts = Vec::with_capacity(labels);
    for _ in 0..labels {
        parts.push(arbitrary_label(u)?);
    }
    Ok(parts.join("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_display() {
        let data = [0u8; 256];
        let mut u = Unstructured::new(&data);
        let a = Addr::arbitrary(&mut u).unwrap();
        // Display output must parse back to something (no panic).
        let _ = Addr::parse(&a.to_string());
    }
}
