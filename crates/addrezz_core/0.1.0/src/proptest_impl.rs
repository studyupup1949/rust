//! `proptest::Arbitrary` strategy for [`Addr`].
//!
//! Sampled `Addr`s are constructed directly from their fields so that
//! they are always well-formed, regardless of shrinking.

use proptest::prelude::*;
use proptest::sample::select;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{Addr, Host, Scheme, Userinfo};

fn scheme_strategy() -> impl Strategy<Value = Scheme> {
    select(vec![
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
    ])
}

// Letter-leading labels ensure the result is accepted as a domain by
// `url::Url` — which rejects some all-numeric or numeric-leading labels
// because they ambiguate with IPv4 literals.
fn label_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,11}".prop_map(|s| s.to_string())
}

fn domain_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(label_strategy(), 2..=4).prop_map(|parts| parts.join("."))
}

fn host_strategy() -> impl Strategy<Value = Host> {
    prop_oneof![
        any::<[u8; 4]>().prop_map(|o| Host::Ipv4(Ipv4Addr::from(o))),
        any::<[u8; 16]>().prop_map(|o| Host::Ipv6(Ipv6Addr::from(o))),
        domain_strategy().prop_map(Host::Domain),
    ]
}

fn userinfo_strategy() -> impl Strategy<Value = Option<Userinfo>> {
    prop::option::of(
        (label_strategy(), prop::option::of(label_strategy()))
            .prop_map(|(username, password)| Userinfo { username, password }),
    )
}

impl Arbitrary for Addr {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        (
            scheme_strategy(),
            userinfo_strategy(),
            host_strategy(),
            prop::option::of(1u16..=65535),
            prop::option::of(label_strategy()),
            prop::option::of(label_strategy()),
            prop::option::of(label_strategy()),
        )
            .prop_map(|(scheme, userinfo, host, port, path, query, fragment)| {
                let path = path.map(|p| format!("/{p}")).unwrap_or_else(|| "/".to_string());
                Addr {
                    scheme,
                    userinfo,
                    host,
                    port,
                    path,
                    query,
                    fragment,
                }
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Addr;

    proptest! {
        #[test]
        fn display_is_nonempty(a in any::<Addr>()) {
            prop_assert!(!a.to_string().is_empty());
        }

        /// Display output must parse, and from the second iteration
        /// onwards Display is a fixed point. The first serialization
        /// may differ from the canonical form because `url::Url`
        /// normalizes (e.g. drops default ports), so we assert the
        /// weaker property `s2 == s3`.
        #[test]
        fn display_is_idempotent(a in any::<Addr>()) {
            let s1 = a.to_string();
            let a2 = Addr::parse(&s1)
                .map_err(|e| TestCaseError::fail(format!("first parse failed for {s1:?}: {e}")))?;
            let s2 = a2.to_string();
            let a3 = Addr::parse(&s2)
                .map_err(|e| TestCaseError::fail(format!("second parse failed for {s2:?}: {e}")))?;
            let s3 = a3.to_string();
            prop_assert_eq!(s2, s3);
        }

        /// Scheme always round-trips exactly.
        #[test]
        fn scheme_roundtrips(a in any::<Addr>()) {
            let parsed = Addr::parse(&a.to_string())
                .map_err(|e| TestCaseError::fail(format!("reparse failed: {e}")))?;
            prop_assert_eq!(a.scheme, parsed.scheme);
        }

        /// The effective port (explicit or scheme-default) is preserved.
        #[test]
        fn effective_port_roundtrips(a in any::<Addr>()) {
            let parsed = Addr::parse(&a.to_string())
                .map_err(|e| TestCaseError::fail(format!("reparse failed: {e}")))?;
            prop_assert_eq!(a.effective_port(), parsed.effective_port());
        }
    }
}
