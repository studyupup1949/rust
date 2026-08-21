//----------------------------------------
// Abstract types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pair {
    #[prost(bytes, tag="1")]
    pub key: std::vec::Vec<u8>,
    #[prost(bytes, tag="2")]
    pub value: std::vec::Vec<u8>,
}
