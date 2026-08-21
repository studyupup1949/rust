//! Constants



/// Arrays of sizes from 0 to 32
pub const NULL_PUBKEY:[u8;32] = [0;32];

/// null address
pub const NULL_ADDRESS:&str = "11111111111111111111111111111111";
/// address of admin 1
pub const ADMIN1:&str = "EnHBYsRckMqtAkpfXhhRTpCo3XF5CFgywXyktqMYoJcV";
/// address of admin 2
pub const ADMIN2:&str= "v5HYHkpee1kgrrCn7xiAfJcfDND6pAX99BoEgxVKfbV";

/// size of accounts for stake nft 
pub const NB_ACCOUNT:usize=8;

/// Public key size 
pub const PUBKEY_SIZE: usize = 32;

/// The buffer of public key of SUPER ADMIN 
pub const BUFFER_PUBKEY_SUPER_ADMIN:[u8;32] = [
    204, 193, 93, 5, 199, 108, 171, 91, 214, 51, 110, 202, 161, 21, 93, 119, 0, 76, 78, 118,
    41, 98, 33, 79, 138, 6, 196, 205, 188, 29, 102, 142,
];