mod error;
mod flags;
mod length;
mod pdu;
mod root_layer;
mod vector;

pub use error::AcnError;
pub use flags::Flags;
pub use length::Length;
pub use pdu::PduCodec;
pub use root_layer::RootLayerCodec;
pub use vector::Vector;

// The ACN identifier is a 12-byte array that identifies the protocol as ACN.
// byte encoded of "ASC-E1.17\0\0\0";
pub const ACN_IDENTIFIER: [u8; 12] = [
    0x41, 0x53, 0x43, 0x2d, 0x45, 0x31, 0x2e, 0x31, 0x37, 0x00, 0x00, 0x00,
];
