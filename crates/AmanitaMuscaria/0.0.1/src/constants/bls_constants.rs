//==========BLS SIGNATURE==========//
// BASICS
    // ALGORITHM: Boneh-Lynn-Shacham (BLS)
    // PK: 48
    // SK: 32
    // SIG: 96
// METADATA
    // VERSION: 0
    // AGGREGATABLE: true
    // IS_POST_QUANTUM_SECURE: false

    pub const BLS_ALGORITHM: &'static str = "Boneh-Lynn-Shacham (BLS)";
    pub const BLS_PK_SIZE_IN_BYTES: usize = 48;
    pub const BLS_SK_SIZE_IN_BYTES: usize = 32;
    pub const BLS_SIGNATURE_SIZE_IN_BYTES: usize = 96;
    
    pub const VERSION: u8 = 0;
    pub const BLS_AGGREGATABLE: bool = true;
    pub const BLS_IS_POST_QUANTUM_SECURE: bool = false;

    pub const BLS_CHARACTER_IN_BETWEEN_MESSAGE_WITH_PK: &'static str = "_";
    
//==========END OF BLS SIGNATURE==========