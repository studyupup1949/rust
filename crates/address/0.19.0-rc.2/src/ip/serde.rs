use crate::{FromStrVisitor, IPAddress, IPv4Address, IPv6Address};
use serde::de::{EnumAccess, Error, Unexpected, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

impl Serialize for IPv4Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            self.address().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for IPv4Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("an IPv4 address string"))
        } else {
            <[u8; 4]>::deserialize(deserializer).map(Self::new)
        }
    }
}

impl Serialize for IPv6Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            self.address().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for IPv6Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("an IPv6 address string"))
        } else {
            <[u8; 16]>::deserialize(deserializer).map(Self::new)
        }
    }
}

/// The `IPAddress` variant identifier for binary formats.
enum IPAddressVariant {
    V4,
    V6,
}

impl IPAddressVariant {
    //! Variant Names

    /// The variant names. (indexed by the variant tag)
    const NAMES: &'static [&'static str] = &["V4", "V6"];
}

impl<'de> Deserialize<'de> for IPAddressVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VariantVisitor;

        impl Visitor<'_> for VariantVisitor {
            type Value = IPAddressVariant;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str("`V4` or `V6`")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    0 => Ok(IPAddressVariant::V4),
                    1 => Ok(IPAddressVariant::V6),
                    _ => Err(E::invalid_value(Unexpected::Unsigned(v), &self)),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    "V4" => Ok(IPAddressVariant::V4),
                    "V6" => Ok(IPAddressVariant::V6),
                    _ => Err(E::unknown_variant(v, IPAddressVariant::NAMES)),
                }
            }
        }

        deserializer.deserialize_identifier(VariantVisitor)
    }
}

impl Serialize for IPAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            match self {
                Self::V4(ip) => serializer.serialize_newtype_variant("IPAddress", 0, "V4", ip),
                Self::V6(ip) => serializer.serialize_newtype_variant("IPAddress", 1, "V6", ip),
            }
        }
    }
}

impl<'de> Deserialize<'de> for IPAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("an IP address string"))
        } else {
            struct IPAddressVisitor;

            impl<'de> Visitor<'de> for IPAddressVisitor {
                type Value = IPAddress;

                fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    f.write_str("an IP address")
                }

                fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                where
                    A: EnumAccess<'de>,
                {
                    match data.variant()? {
                        (IPAddressVariant::V4, variant) => {
                            variant.newtype_variant::<IPv4Address>().map(IPAddress::V4)
                        }
                        (IPAddressVariant::V6, variant) => {
                            variant.newtype_variant::<IPv6Address>().map(IPAddress::V6)
                        }
                    }
                }
            }

            deserializer.deserialize_enum("IPAddress", IPAddressVariant::NAMES, IPAddressVisitor)
        }
    }
}
