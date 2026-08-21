use crate::{
    FromStrVisitor, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4,
    SocketAddressV6,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for SocketAddressV4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            (self.ip(), self.port()).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for SocketAddressV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("an IPv4 socket address string"))
        } else {
            <(IPv4Address, u16)>::deserialize(deserializer).map(|(ip, port)| Self::new(ip, port))
        }
    }
}

impl Serialize for SocketAddressV6 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            (self.ip(), self.port()).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for SocketAddressV6 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("an IPv6 socket address string"))
        } else {
            <(IPv6Address, u16)>::deserialize(deserializer).map(|(ip, port)| Self::new(ip, port))
        }
    }
}

impl Serialize for SocketAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            (self.ip(), self.port()).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for SocketAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FromStrVisitor::new("a socket address string"))
        } else {
            <(IPAddress, u16)>::deserialize(deserializer).map(|(ip, port)| Self::new(ip, port))
        }
    }
}
