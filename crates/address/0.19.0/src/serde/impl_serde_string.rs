/// Implements `Serialize` and `Deserialize` for an owned type that serializes as its `Display` string.
macro_rules! impl_serde_string {
    ($ty:ident, $expecting:literal) => {
        impl ::serde::Serialize for crate::$ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for crate::$ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(crate::serde::FromStrVisitor::new($expecting))
            }
        }
    };
}

/// Implements `Serialize` and `Deserialize` for a reference type that serializes as its `Display` string.
macro_rules! impl_serde_string_ref {
    ($ty:ident, $owned:ident, $expecting:literal) => {
        impl<'a> ::serde::Serialize for crate::$ty<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        impl<'de: 'a, 'a> ::serde::Deserialize<'de> for crate::$ty<'a> {
            #[doc = concat!(
                "The string is borrowed from the input, so domain names must be lowercase and ",
                "must not contain escape sequences. Use [`",
                stringify!($owned),
                "`](crate::",
                stringify!($owned),
                ") to deserialize mixed-case or escaped input."
            )]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let visitor = crate::serde::TryFromStrVisitor::new($expecting);
                let value: crate::$ty<'de> = deserializer.deserialize_str(visitor)?;
                Ok(value)
            }
        }
    };
}

impl_serde_string!(Authority, "an authority string");
impl_serde_string_ref!(AuthorityRef, Authority, "a borrowed authority string");

impl_serde_string!(Domain, "a domain string");
impl_serde_string_ref!(DomainRef, Domain, "a borrowed domain string");

impl_serde_string!(Endpoint, "an endpoint string");
impl_serde_string_ref!(EndpointRef, Endpoint, "a borrowed endpoint string");

impl_serde_string!(Host, "a host string");
impl_serde_string_ref!(HostRef, Host, "a borrowed host string");
