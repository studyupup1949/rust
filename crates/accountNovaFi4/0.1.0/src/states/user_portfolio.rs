//! State transition types
use arrayref::{array_ref, array_refs};
use serde::{Deserialize, Serialize};
use solana_program::{
    entrypoint_deprecated::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};
use std::vec::Vec;

use crate::{
    constants::{
        account_size::{SPLU_LEN, USER_PORTFOLIO_PREFIX},
        constant::{NULL_PUBKEY, PUBKEY_SIZE},
        account_type::TYPE_ACCOUNT_USER_PORTFOLIO_ACCOUNT,
    },
    error::PortfolioError,
};

const TYPE_SIZE: usize = 1;
const STATE_SIZE: usize = 1;
const AMOUNT_SIZE: usize = 8;
const VERSION_SIZE: usize = 1;
const NONCE_SIZE: usize = 1;
const STATE_SPLU_SECONDARY: usize = 1;
const STATE_SPLU_TERTIARY1: usize = 1;
const STATE_SPLU_TERTIARY2: usize = 1;
const NONCE_SPLU: usize = 1;

/// struct of SPLU of PPU
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpluStruct {
    /// SPLU secondary address.
    pub splu_secondary: Pubkey,
    /// State of SPLU secondary.
    pub state_splu_secondary: u8,
    /// First SPLU tertiary address.
    pub splu_tertiary1: Pubkey,
    /// State of first SPLU tertiary.
    pub state_splu_tertiary1: u8,
    /// Second SPLU tertiary address.
    pub splu_tertiary2: Pubkey,
    /// State of second SPLU tertiary.
    pub state_splu_tertiary2: u8,
    /// Authority of SPLU.
    pub authority_splu: Pubkey,
    /// Account_program of SPLU.
    pub program_account_splu: Pubkey,
    /// Nonce of SPLU.
    pub nonce_splu: u8,
}

impl SpluStruct {
    /// Create new SPLU Struct
    pub fn add_new_splu_secondary(
        &mut self,
        splu_secondary_address: Pubkey,
        authority_splu: Pubkey,
        program_account_splu: Pubkey,
        nonce_splu: u8,
    ) -> ProgramResult {
        self.splu_secondary = splu_secondary_address;
        self.state_splu_secondary = 1;
        self.authority_splu = authority_splu;
        self.program_account_splu = program_account_splu;
        self.nonce_splu = nonce_splu;

        Ok(())
    }

    /// update the state of splu secondary
    pub fn update_splu_secondary(&mut self) -> ProgramResult {
        self.state_splu_secondary = self
            .state_splu_secondary
            .checked_add(1)
            .ok_or(PortfolioError::InvalidAmount)?;

        Ok(())
    }

    /// reset the state of splu secondary
    pub fn reset_splu_secondary(&mut self) -> ProgramResult {
        self.state_splu_secondary = 0;

        Ok(())
    }

    /// add new splu teriary
    pub fn add_splu_tertiary(
        &mut self,
        splu_tertiary1: Pubkey,
        splu_tertiary2: Pubkey,
    ) -> ProgramResult {
        self.splu_tertiary1 = splu_tertiary1;
        self.state_splu_tertiary1 = self
            .state_splu_tertiary1
            .checked_add(1)
            .ok_or(PortfolioError::InvalidAmount)?;
        self.splu_tertiary2 = splu_tertiary2;
        if splu_tertiary2.as_ref() != NULL_PUBKEY {
            self.state_splu_tertiary2 = self
                .state_splu_tertiary2
                .checked_add(1)
                .ok_or(PortfolioError::InvalidAmount)?;
        }
        Ok(())
    }

    /// reset the state of splu teriary
    pub fn reset_splu_tertiary(&mut self) -> ProgramResult {
        self.state_splu_tertiary1 = 0;
        self.state_splu_tertiary2 = 0;

        Ok(())
    }

    /// update the state splu teriary
    pub fn update_state_tertiary(&mut self, state1: u8, state2: u8) -> ProgramResult {
        self.state_splu_tertiary1 = state1;
        self.state_splu_tertiary2 = state2;

        Ok(())
    }
}

///  User Portfolio data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]

pub struct UserPortfolio {
    /// The type of account.
    pub type_account: u8,
    /// The owner of user portfolio.
    pub owner: Pubkey,
    /// Portfolio depends of user portfolio.
    pub portfolio_address: Pubkey,
    /// State of PPU to know which step was completed.
    pub state: u8,
    /// Last amount deposit or withraw.
    pub amount: u64,
    /// Version of PPU struct.
    pub version: u8,
    /// Account to be used to save extra data.
    pub extended_data: Pubkey,
    /// Splm primary which user deposit from.
    pub splm_primary: Pubkey,
    /// Authority of PPU.
    pub authority: Pubkey,
    /// Nonce used in cross program.
    pub nonce: u8,
    /// Create account program used in cross program.
    pub create_account_program: Pubkey,
    /// List of splu contain the amount transfered after swap.
    pub splu_list: Vec<SpluStruct>,
}

impl Sealed for UserPortfolio {}

impl IsInitialized for UserPortfolio {
    fn is_initialized(&self) -> bool {
        return self.type_account == TYPE_ACCOUNT_USER_PORTFOLIO_ACCOUNT;
    }
}

/// pack and unpack for user portfolio
pub trait PackUserPortfolio {
    /// unpack user portfolio
    fn unpack_user_portfolio(src: &[u8]) -> Result<UserPortfolio, ProgramError>;
    /// pack user portfolio
    fn pack_user_portfolio(&self, dst: &mut [u8]);
}

impl PackUserPortfolio for UserPortfolio {
    /// unpack user portfolio
    fn unpack_user_portfolio(src: &[u8]) -> Result<Self, ProgramError> {
        let numbre_splu = (&src.len() - USER_PORTFOLIO_PREFIX) / SPLU_LEN;
        let len_splu_data = &src.len() - USER_PORTFOLIO_PREFIX;

        let src_fix = array_ref![&src, 0, USER_PORTFOLIO_PREFIX];
        let (
            type_account,
            owner,
            portfolio_address,
            state,
            amount,
            version,
            extended_data,
            splm_primary,
            authority,
            nonce,
            create_account_program,
        ) = array_refs![
            src_fix,
            TYPE_SIZE,
            PUBKEY_SIZE,
            PUBKEY_SIZE,
            STATE_SIZE,
            AMOUNT_SIZE,
            VERSION_SIZE,
            PUBKEY_SIZE,
            PUBKEY_SIZE,
            PUBKEY_SIZE,
            NONCE_SIZE,
            PUBKEY_SIZE
        ];

        let mut splu_vec: Vec<SpluStruct> = Vec::with_capacity(numbre_splu);

        let list_splu_data =
            &src[USER_PORTFOLIO_PREFIX..USER_PORTFOLIO_PREFIX + (len_splu_data) as usize];

        // unpack list of splu struct
        let mut offset = 0;
        for _ in 0..numbre_splu {
            let splu_data = array_ref![list_splu_data, offset, SPLU_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (
                splu_secondary,
                state_splu_secondary,
                splu_tertiary1,
                state_splu_tertiary1,
                splu_tertiary2,
                state_splu_tertiary2,
                authority_splu,
                program_account_splu,
                nonce_splu,
            ) = array_refs![
                splu_data,
                PUBKEY_SIZE,
                STATE_SPLU_SECONDARY,
                PUBKEY_SIZE,
                STATE_SPLU_TERTIARY1,
                PUBKEY_SIZE,
                STATE_SPLU_TERTIARY2,
                PUBKEY_SIZE,
                PUBKEY_SIZE,
                NONCE_SPLU
            ];
            splu_vec.push(SpluStruct {
                splu_secondary: Pubkey::new_from_array(*splu_secondary),
                state_splu_secondary: u8::from_le_bytes(*state_splu_secondary),
                splu_tertiary1: Pubkey::new_from_array(*splu_tertiary1),
                state_splu_tertiary1: u8::from_le_bytes(*state_splu_tertiary1),
                splu_tertiary2: Pubkey::new_from_array(*splu_tertiary2),
                state_splu_tertiary2: u8::from_le_bytes(*state_splu_tertiary2),
                authority_splu: Pubkey::new_from_array(*authority_splu),
                program_account_splu: Pubkey::new_from_array(*program_account_splu),
                nonce_splu: u8::from_le_bytes(*nonce_splu),
            });
            offset += SPLU_LEN;
        }

        Ok(UserPortfolio {
            type_account: u8::from_le_bytes(*type_account),
            owner: Pubkey::new_from_array(*owner),
            portfolio_address: Pubkey::new_from_array(*portfolio_address),
            state: u8::from_le_bytes(*state),
            amount: u64::from_le_bytes(*amount),
            version: u8::from_le_bytes(*version),
            extended_data: Pubkey::new_from_array(*extended_data),
            splm_primary: Pubkey::new_from_array(*splm_primary),
            authority: Pubkey::new_from_array(*authority),
            nonce: u8::from_le_bytes(*nonce),
            create_account_program: Pubkey::new_from_array(*create_account_program),
            splu_list: splu_vec.to_vec(),
        })
    }

    /// pack user portfolio
    fn pack_user_portfolio(&self, dst: &mut [u8]) {
        let UserPortfolio {
            type_account,
            owner,
            portfolio_address,
            state,
            amount,
            version,
            extended_data,
            splm_primary,
            authority,
            nonce,
            create_account_program,
            splu_list,
        } = self;
        let mut buffer = [0; USER_PORTFOLIO_PREFIX + (SPLU_LEN * 10)]; // MAXIMUM 10 SPLU

        buffer[0] = *type_account;
        let owner_range = TYPE_SIZE..TYPE_SIZE + PUBKEY_SIZE;
        buffer[owner_range].clone_from_slice(owner.as_ref());
        let portfolio_address_range =
            TYPE_SIZE + PUBKEY_SIZE..TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE;
        buffer[portfolio_address_range].clone_from_slice(portfolio_address.as_ref());
        let state_range = TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE;
        buffer[state_range] = *state;
        let amount_range = TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE + STATE_SIZE
            ..TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE + STATE_SIZE + AMOUNT_SIZE;
        let amount_to_array = amount.to_le_bytes();
        buffer[amount_range].clone_from_slice(&amount_to_array[0..]);
        let version_range = TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE + STATE_SIZE + AMOUNT_SIZE;
        buffer[version_range] = *version;
        let extended_data_range =
            TYPE_SIZE + PUBKEY_SIZE + PUBKEY_SIZE + STATE_SIZE + AMOUNT_SIZE + VERSION_SIZE
                ..TYPE_SIZE
                    + PUBKEY_SIZE
                    + PUBKEY_SIZE
                    + STATE_SIZE
                    + AMOUNT_SIZE
                    + VERSION_SIZE
                    + PUBKEY_SIZE;
        buffer[extended_data_range].clone_from_slice(extended_data.as_ref());
        let splm_primary_range = TYPE_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + STATE_SIZE
            + AMOUNT_SIZE
            + VERSION_SIZE
            + PUBKEY_SIZE
            ..TYPE_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + STATE_SIZE
                + AMOUNT_SIZE
                + VERSION_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE;
        buffer[splm_primary_range].clone_from_slice(splm_primary.as_ref());
        let authority_range = TYPE_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + STATE_SIZE
            + AMOUNT_SIZE
            + VERSION_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            ..TYPE_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + STATE_SIZE
                + AMOUNT_SIZE
                + VERSION_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE;
        buffer[authority_range].clone_from_slice(authority.as_ref());
        let nonce_range = TYPE_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + STATE_SIZE
            + AMOUNT_SIZE
            + VERSION_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE;
        buffer[nonce_range] = *nonce;
        let create_account_program_range = TYPE_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + STATE_SIZE
            + AMOUNT_SIZE
            + VERSION_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + PUBKEY_SIZE
            + NONCE_SIZE
            ..TYPE_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + STATE_SIZE
                + AMOUNT_SIZE
                + VERSION_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE
                + NONCE_SIZE
                + PUBKEY_SIZE;
        buffer[create_account_program_range].clone_from_slice(create_account_program.as_ref());

        let splu_vec_tmp = bincode::serialize(&splu_list).unwrap();

        let mut splu_data_tmp = [0; SPLU_LEN * 10]; // maximum 10 splu

        let len_splu = splu_vec_tmp.len();
        let nbre_splu = len_splu / SPLU_LEN;
        let len_slu_reel = nbre_splu * SPLU_LEN;

        splu_data_tmp[0..len_slu_reel as usize].clone_from_slice(&splu_vec_tmp[8..len_splu]); // 0..8 : 8 bytes contain length of struct
        buffer[USER_PORTFOLIO_PREFIX..USER_PORTFOLIO_PREFIX + len_slu_reel as usize]
            .clone_from_slice(&splu_data_tmp[0..len_slu_reel as usize]);

        dst[0..USER_PORTFOLIO_PREFIX + len_slu_reel as usize]
            .copy_from_slice(&buffer[0..USER_PORTFOLIO_PREFIX + len_slu_reel as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_user_portfolio() {
        let splu_struct = SpluStruct {
            splu_secondary: Pubkey::new_unique(),
            state_splu_secondary: 1,
            splu_tertiary1: Pubkey::new_unique(),
            state_splu_tertiary1: 1,
            splu_tertiary2: Pubkey::new_unique(),
            state_splu_tertiary2: 1,
            authority_splu: Pubkey::new_unique(),
            program_account_splu: Pubkey::new_unique(),
            nonce_splu: 1,
        };

        let mut splu_list = Vec::new();
        splu_list.push(splu_struct);

        let user_portfolio = UserPortfolio {
            type_account: TYPE_ACCOUNT_USER_PORTFOLIO_ACCOUNT,
            owner: Pubkey::new_unique(),
            portfolio_address: Pubkey::new_unique(),
            state: 0,
            amount: 3,
            version: 1,
            extended_data: Pubkey::new_unique(),
            splm_primary: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
            nonce: 1,
            create_account_program: Pubkey::new_unique(),
            splu_list,
        };
        const LEN: usize = USER_PORTFOLIO_PREFIX + (SPLU_LEN * 1); // 204: @préfix + 164:splu_len * 1
        let mut packed = [0u8; LEN];
        UserPortfolio::pack_user_portfolio(&user_portfolio, &mut packed[..]);
        let unpacked = UserPortfolio::unpack_user_portfolio(&packed).unwrap();
        assert_eq!(user_portfolio, unpacked);
        assert_eq!(unpacked.type_account, TYPE_ACCOUNT_USER_PORTFOLIO_ACCOUNT);
    }
}
