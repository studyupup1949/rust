//==========BLS SIGNATURE==========//
// BASICS
    // ALGORITHM: Boneh-Lynn-Shacham (BLS)
    // PK: 48 bytes
    // SK: 32 bytes
    // SIG: 96 bytes
// BASICS HEX-ENCODING
    // PK: 96 bytes
    // SK: 64 bytes
// METADATA
    // VERSION: 0
    // AGGREGATABLE: true
    // IS_POST_QUANTUM_SECURE: false

    pub const BLS_ALGORITHM: &'static str = "Boneh-Lynn-Shacham (BLS)";
    pub const BLS_PK_SIZE_IN_BYTES: usize = 48;
    pub const BLS_SK_SIZE_IN_BYTES: usize = 32;
    pub const BLS_SIGNATURE_SIZE_IN_BYTES: usize = 96;

    pub const BLS_HEX_PK_SIZE_IN_BYTES: usize = 96;
    pub const BLS_HEX_SK_SIZE_IN_BYTES: usize = 64;
    
    pub const VERSION: u8 = 0;
    
    pub const BLS_AGGREGATABLE: bool = true;
    pub const BLS_IS_POST_QUANTUM_SECURE: bool = false;
    
    /// Character that splits the public key and message with signing message with prepended public key
    pub const BLS_CHARACTER_IN_BETWEEN_MESSAGE_WITH_PK: &'static str = "_";

    // EXAMPLES
    pub const BLS_PREPENDED_KEY_EXAMPLE: &'static str = "97847234E2B1365CB33F0C038A604D2CA6E4FE9C4E7CCC2923988E50C07145C5DEB3C561D7F7ADEFCC013F4D5DE3F16A_SignedMessage";

//==========END OF BLS SIGNATURE==========