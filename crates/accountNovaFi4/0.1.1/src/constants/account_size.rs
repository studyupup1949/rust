//! The size of accounts

/// The size of NFT account 
pub const NFT_ACCOUNT_SIZE:usize = 99;



// Distribut account constants.
/// The size of distribute token account 
pub const DISTRIBUTE_TOKEN_SIZE:usize = 1882;
/// Len of constant prefix of distribut account.
pub const DISTRIBUTE_TOKEN_PREFIX_SIZE: usize = 34;
/// Maximum size of list users added to distribut account.
pub const LIST_DISTRIBUTE_USER_SIZE: usize = 1848;
/// Maximum number of users added to distribut account.
pub const NUMBER_USERS_DISTRIBUTE: usize = 44;
/// The size of user distribut struct.
pub const USER_DISTRIBUTE_SIZE: usize = 42;


// Admin account constants.
/// The size of Admin account.
pub const ADMIN_ACCOUNT_SIZE: usize = 331;
/// LEN of constant prefix of admin account.
pub const ADMIN_ACCOUNT_PREFIX_SIZE:usize = 1;
/// The size of user admin struct.
pub const USER_ADMIN_SIZE: usize = 33;
/// The maximum number of users admins can added to admin account.
pub const NUMBER_OF_ADMINS: usize = 10;


// Portfolio account constatnts.
/// LEN of constant prefix of portfolio account.
pub const PORTFOLIO_PREFIX: usize = 245;
/// The size of asset portfolio struct.
pub const ASSET_LEN: usize = 67;



// User Portfolio account constatnts.
/// LEN of constant prefix of user portfolio account.
pub const USER_PORTFOLIO_PREFIX: usize = 204;
/// The size of splu user portfolio struct.
pub const SPLU_LEN: usize = 164;