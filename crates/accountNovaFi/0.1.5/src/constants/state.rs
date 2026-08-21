//! the state of instructions

/// State when user portfolio account created.
pub const CREATE_PPU_STATE:u8 = 0;

/// State ppu upon the termination of the deposit in the portfolio.
pub const DEPOSIT_PORTFOLIO_STATE :u8= 1;

/// State ppu upon the termination of the deposit into lp token. 
pub const DEPOSIT_INTO_LP_STATE :u8= 2;

/// State ppu upon the termination of the stake in splu LP token.
pub const STAKE_PORTFOLIO_STATE :u8 = 3;

/// State ppu when invest more step terminated.
pub const INVEST_MORE_PORTFOLIO_STATE :u8= 8;

/// State ppu when invest more into lp step terminated.
pub const INVEST_MORE_INTO_LP_TOKEN_STATE :u8= 9;

/// State ppu when stake more in splu lp step terminated.
pub const INVEST_MORE_STAKE_STATE:u8=10;

/// State ppu when Stop Staking. 
pub const WITHDRAW_FROM_QUARRY_STATE :u8 = 4 ;

/// State ppu when exit yield protocol.
pub const WITHDRAW_FROM_SABER_STATE  :u8= 5 ;

/// State ppu when exit from portfolio.
pub const WITHDRAW_FROM_PORTFOLIO_STATE :u8 = 6;

/// State ppu when closed user portfolio account.
pub const CLOSE_PPU_STATE :u8= 7; 

/// States of is-initialize of portfolio account.
/// State of ppm while creating.
pub const PPM_INITIALIZED: u8 = 1;

/// State of ppm when complet adding assets.
pub const PPM_COMPLETED: u8 = 2;

/// State of ppm when owner complet all step of join.
pub const PPM_JOINED: u8 = 3;


/// NFT Token
/// The user wasn't staked his nft token.
pub const NFT_NOT_STAKED:u8 = 0;

/// The user was staked his nft token.
pub const NFT_STAKED:u8 = 1;

