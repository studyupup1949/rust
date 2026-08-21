use crate::serde::FromStrVisitor;
use crate::{IPAddress, IPv4Address, IPv6Address};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

impl Serialize for IPAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            match self {
                Self::V4(ip) => serializer.serialize_bytes(&ip.address()),
                Self::V6(ip) => serializer.serialize_bytes(&ip.address()),
            }
        }
    }
}

/// A serde visitor that matches a byte string's length: 4 bytes for an IPv4 address, 16 bytes for an IPv6 address.
struct IPAddressBytesVisitor;

impl<'de> Visitor<'de> for IPAddressBytesVisitor {
    type Value = IPAddress;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("4 or 16 IP address bytes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(address) = <[u8; 4]>::try_from(v) {
            Ok(IPv4Address::new(address).to_ip())
        } else if let Ok(address) = <[u8; 16]>::try_from(v) {
            Ok(IPv6Address::new(address).to_ip())
        } else {
            Err(E::invalid_length(v.len(), &self))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut address: [u8; 16] = [0; 16];
        let mut len: usize = 0;
        while let Some(byte) = seq.next_element::<u8>()? {
            if len == address.len() {
                return Err(A::Error::invalid_length(len + 1, &self));
            }
            address[len] = byte;
            len += 1;
        }
        self.visit_bytes(&address[..len])
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
            deserializer.deserialize_bytes(IPAddressBytesVisitor)
        }
    }
}
