//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::{ProgramError, PrintProgramError}, msg};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PortfolioError {
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// Insufficient funds for the operation requested.
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// Invalid Mint.
    #[error("Invalid Mint")]
    InvalidMint,
    /// Account not associated with this Mint.
    #[error("Account not associated with this Mint")]
    MintMismatch,
    /// Owner does not match.
    #[error("Owner does not match")]
    OwnerMismatch,
    /// This token's supply is fixed and new tokens cannot be minted.
    #[error("Fixed supply")]
    FixedSupply,
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// Invalid number of provided signers.
    #[error("Invalid number of provided signers")]
    InvalidNumberOfProvidedSigners,
    /// Invalid number of required signers.
    #[error("Invalid number of required signers")]
    InvalidNumberOfRequiredSigners,
    /// State is uninitialized.
    #[error("State is unititialized")]
    UninitializedState,
    /// Instruction does not support native tokens
    #[error("Instruction does not support native tokens")]
    NativeNotSupported,
    /// Non-native account can only be closed if its balance is zero
    #[error("Non-native account can only be closed if its balance is zero")]
    NonNativeHasBalance,
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// State is invalid for requested operation.
    #[error("State is invalid for requested operation")]
    InvalidState,
    /// Operation overflowed
    #[error("Operation overflowed")]
    Overflow,
    /// Account does not support specified authority type.
    #[error("Account does not support specified authority type")]
    AuthorityTypeNotSupported,
    /// This token mint cannot freeze accounts.
    #[error("This token mint cannot freeze accounts")]
    MintCannotFreeze,
    /// Account is frozen; all account operations will fail
    #[error("Account is frozen")]
    AccountFrozen,
    /// Mint decimals mismatch between the client and mint
    #[error("The provided decimals value different from the Mint decimals")]
    MintDecimalsMismatch,
    /// Invalid amount, must be greater then zero
    #[error("Input amount is invalid")]
    InvalidAmount,
    /// Invalid type of account
    #[error("Invalid type account")]
    InvalidTypeAccount,
    /// Error while create user portfolio
    #[error("Error while create user portfolio")]
    ErrorWhileCreatePPU,
    /// Maximum number of splu attended
    #[error("Exceeded numbre of splu")]
    MaximumSPLUAdded,
    /// Maximum number of admins attended
    #[error("Exceeded numbre of admins")]
    MaximumADMINAdded,
    /// Missing admin authorization
    #[error("Missing admin authorization")]
    MissingAdminAuthorization,
    /// Error while adding new asset to portfolio
    #[error("Error while adding new asset to portfolio")]
    ErrorWhileAddingNewAssetToPortfolio,
    /// Error while adding new splu to user portfolio
    #[error("Error while adding new splu to user portfolio")]
    ErrorWhileAddingNewSpluToUserPortfolio,
    /// Error while adding new ppu to distribute account
    #[error("Maximum number of ppu was attended")]
    MaximumNumberOfPPUWasAttended,
    /// Error invalid state of user portfolio account
    #[error("Error invalid state of user portfolio account")]
    InvalidStateOfPPU,
    /// Error invalid state of portfolio account
    #[error("Error invalid state of portfolio account")]
    InvalidStateOfPPM,
    /// Error while updating data account
    #[error("Error while updating data account")]
    ErrorWhileUpdatingDataAccount,
    /// Error while executing external transaction
    #[error("Error while executing an external transaction")]
    ErrorFromExternalTransaction,
    
}
impl From<PortfolioError> for ProgramError {
    fn from(e: PortfolioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for PortfolioError {
    fn type_of() -> &'static str {
        "PortfolioError"
    }
}



impl PrintProgramError for PortfolioError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            PortfolioError::NotRentExempt => {
                msg!("Error: Lamport balance below rent-exempt threshold")
            }
            PortfolioError::InsufficientFunds => msg!("Error: insufficient funds"),
            PortfolioError::InvalidMint => msg!("Error: Invalid Mint"),
            PortfolioError::MintMismatch => msg!("Error: Account not associated with this Mint"),
            PortfolioError::OwnerMismatch => msg!("Error: owner does not match"),
            PortfolioError::FixedSupply => msg!("Error: the total supply of this token is fixed"),
            PortfolioError::AlreadyInUse => msg!("Error: account or token already in use"),
            PortfolioError::InvalidNumberOfProvidedSigners => {
                msg!("Error: Invalid number of provided signers")
            }
            PortfolioError::InvalidNumberOfRequiredSigners => {
                msg!("Error: Invalid number of required signers")
            }
            PortfolioError::UninitializedState => msg!("Error: State is uninitialized"),
            PortfolioError::NativeNotSupported => {
                msg!("Error: Instruction does not support native tokens")
            }
            PortfolioError::NonNativeHasBalance => {
                msg!("Error: Non-native account can only be closed if its balance is zero")
            }
            PortfolioError::InvalidInstruction => msg!("Error: Invalid instruction"),
            PortfolioError::InvalidState => msg!("Error: Invalid account state for operation"),
            PortfolioError::Overflow => msg!("Error: Operation overflowed"),
            PortfolioError::AuthorityTypeNotSupported => {
                msg!("Error: Account does not support specified authority type")
            }
            PortfolioError::MintCannotFreeze => {
                msg!("Error: This token mint cannot freeze accounts")
            }
            PortfolioError::AccountFrozen => msg!("Error: Account is frozen"),
            PortfolioError::MintDecimalsMismatch => {
                msg!("Error: decimals different from the Mint decimals")
            }
            PortfolioError::InvalidAmount => {
                msg!("Error: invalid amount ")
            }
            PortfolioError::InvalidTypeAccount => {
                msg!("Error: Invalid type account ")
            }
            PortfolioError::ErrorWhileCreatePPU => {
                msg!("Error: Error while create user portfolio ")
            }

            PortfolioError::MaximumSPLUAdded => {
                msg!("Error: Maximum number of splu attended ")
            }
            PortfolioError::MaximumADMINAdded => {
                msg!("Error: Maximum number of admins attended ")
            }
            PortfolioError::MissingAdminAuthorization => {
                msg!("Error: Missing admin authorization ")
            }
            PortfolioError::ErrorWhileAddingNewAssetToPortfolio => {
                msg!("Error: Error while adding new asset to portfolio ")
            }
            PortfolioError::ErrorWhileAddingNewSpluToUserPortfolio => {
                msg!("Error: Error while adding new splu to user portfolio ")
            }
            PortfolioError::MaximumNumberOfPPUWasAttended => {
                msg!("Error: Maximum number of ppu was attended ")
            }
            PortfolioError:: InvalidStateOfPPU => {
                msg!("Error invalid state of user portfolio account")
            }
            PortfolioError:: InvalidStateOfPPM => {
                msg!("Error invalid state of portfolio account")
            }
            PortfolioError:: ErrorWhileUpdatingDataAccount => {
                msg!("Error Error while updating data account")
            }
            PortfolioError:: ErrorFromExternalTransaction => {
                msg!("Error while executing an external transaction")
            }
             
        }
    }
}
