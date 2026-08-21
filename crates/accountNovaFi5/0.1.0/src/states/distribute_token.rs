//! State transition types
use crate::constants::account_size::{DISTRIBUTE_TOKEN_PREFIX_SIZE, NUMBER_USERS_DISTRIBUTE, LIST_DISTRIBUTE_USER_SIZE, USER_DISTRIBUTE_SIZE};
use crate::constants::constant::PUBKEY_SIZE;
use crate::{
    constants::{account_size::DISTRIBUTE_TOKEN_SIZE, account_type::TYPE_ACCOUNT_AIRDROP_ACCOUNT},
    error::PortfolioError,
};
use arrayref::{array_ref, array_refs};
use serde::{Deserialize, Serialize};
use solana_program::{
    entrypoint_deprecated::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};
use std::vec::Vec;

const TYPE_SIZE: usize = 1;
const STATE_DISTIBUTE_TOKENS_SIZE: usize = 1;
const AMOUNT_SIZE: usize = 8;
const NFT_FIND_SIZE: usize = 1;
const STATE_USER_DISTRIBUTE_SIZE: usize = 1;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]

pub struct UserDistributeStruct {
    /// The public address owned by user .
    pub user_portfolio_account: Pubkey,
    ///  The last amount deposit or withraw.
    pub amount: u64,
    /// _0. if he wasn't staked an nft token.
    /// _1. if he was staked an nft token.
    pub nft_find: u8,
    /// state of the user portfolio.
    pub state_user_distribute: u8,
}
impl UserDistributeStruct {
    /// add user distribute
    pub fn add_user_distribute(
        &mut self,
        user_portfolio_account: Pubkey,
        amount: u64,
        nft_find: u8,
        state_user_distribute: u8,
    ) -> ProgramResult {
        self.user_portfolio_account = user_portfolio_account;
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(PortfolioError::InvalidAmount)?;

        self.nft_find = nft_find;
        self.state_user_distribute = state_user_distribute;
        Ok(())
    }
    /// update the state of  user distribute
    pub fn update_state_user_distribute(&mut self, state_user_distribute: u8) -> ProgramResult {
        self.state_user_distribute = state_user_distribute;
        Ok(())
    }
}

///  distribute account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DistributeTokens {
    /// The type of account .
    pub type_account: u8,
    /// state of the user portfolio.
    pub state_distribute_tokens: u8,
    /// The portfolio depends of distribute token.
    pub portfolio_address: Pubkey,
    /// list of [`UserDistributeStruct`]
    pub list_user_distribute: Vec<UserDistributeStruct>,
}

impl Sealed for DistributeTokens {}

impl IsInitialized for DistributeTokens {
    fn is_initialized(&self) -> bool {
        return self.type_account == TYPE_ACCOUNT_AIRDROP_ACCOUNT;
    }
}
/// pack and unpack for distribute token
pub trait PackDistributeTokens {
    /// unpack distribute token
    fn unpack_distribute_token(src: &[u8]) -> Result<DistributeTokens, ProgramError>;
    /// pack diestribute token
    fn pack_distribute_token(&self, dst: &mut [u8]);
}

impl PackDistributeTokens for DistributeTokens {
    /// unpack distribute token
    fn unpack_distribute_token(src: &[u8]) -> Result<Self, ProgramError> {
        let src_fix = array_ref![&src, 0, DISTRIBUTE_TOKEN_PREFIX_SIZE];
        let (type_account, state_distribute_tokens, portfolio_address) =
            array_refs![src_fix, TYPE_SIZE, STATE_DISTIBUTE_TOKENS_SIZE, PUBKEY_SIZE];
        let mut user_distribute_vec: Vec<UserDistributeStruct> =
            Vec::with_capacity(NUMBER_USERS_DISTRIBUTE);

        // unpack the list of user distribure
        let list_user_distribute_data = &src[DISTRIBUTE_TOKEN_PREFIX_SIZE
            ..DISTRIBUTE_TOKEN_PREFIX_SIZE + LIST_DISTRIBUTE_USER_SIZE as usize];
        let mut offset = 0;
        for _ in 0..NUMBER_USERS_DISTRIBUTE {
            let user_data = array_ref![list_user_distribute_data, offset, USER_DISTRIBUTE_SIZE];
            #[allow(clippy::ptr_offset_with_cast)]
            let (user_portfolio_account, amount, nft_find, state_user_distribute) = array_refs![
                user_data,
                PUBKEY_SIZE,
                AMOUNT_SIZE,
                NFT_FIND_SIZE,
                STATE_USER_DISTRIBUTE_SIZE
            ];
            user_distribute_vec.push(UserDistributeStruct {
                user_portfolio_account: Pubkey::new_from_array(*user_portfolio_account),
                amount: u64::from_le_bytes(*amount),
                nft_find: u8::from_le_bytes(*nft_find),
                state_user_distribute: u8::from_le_bytes(*state_user_distribute),
            });
            offset += USER_DISTRIBUTE_SIZE;
        }

        Ok(DistributeTokens {
            type_account: u8::from_le_bytes(*type_account),
            state_distribute_tokens: u8::from_le_bytes(*state_distribute_tokens),
            portfolio_address: Pubkey::new_from_array(*portfolio_address),
            list_user_distribute: user_distribute_vec.to_vec(),
        })
    }

    /// pack distribute token
    fn pack_distribute_token(&self, dst: &mut [u8]) {
        let DistributeTokens {
            type_account,
            state_distribute_tokens,
            portfolio_address,
            list_user_distribute,
        } = self;

        let mut buffer = [0; DISTRIBUTE_TOKEN_SIZE];
        buffer[0] = *type_account;
        let state_distribute_tokens_range = TYPE_SIZE;
        buffer[state_distribute_tokens_range] = *state_distribute_tokens;
        let portfolio_address_range = TYPE_SIZE + STATE_DISTIBUTE_TOKENS_SIZE
            ..TYPE_SIZE + STATE_DISTIBUTE_TOKENS_SIZE + PUBKEY_SIZE;
        buffer[portfolio_address_range].clone_from_slice(portfolio_address.as_ref());
        let user_distribute_vec_tmp = bincode::serialize(&list_user_distribute).unwrap();
        let mut user_distribute_data_tmp = [0; LIST_DISTRIBUTE_USER_SIZE]; // maximum 44
        let len_user_distribute = user_distribute_vec_tmp.len();
        let nbre_user_distribute = len_user_distribute / USER_DISTRIBUTE_SIZE; //44
        let len_slu_reel = nbre_user_distribute * USER_DISTRIBUTE_SIZE;
        user_distribute_data_tmp[0..len_slu_reel as usize]
            .clone_from_slice(&user_distribute_vec_tmp[8..len_user_distribute]); // 0..8 : 8 bytes contain length of struct

        buffer[DISTRIBUTE_TOKEN_PREFIX_SIZE..DISTRIBUTE_TOKEN_PREFIX_SIZE + len_slu_reel as usize]
            .clone_from_slice(&user_distribute_data_tmp[0..len_slu_reel as usize]);

        dst[0..DISTRIBUTE_TOKEN_PREFIX_SIZE + len_slu_reel as usize]
            .copy_from_slice(&buffer[0..DISTRIBUTE_TOKEN_PREFIX_SIZE + len_slu_reel as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::account_type::TYPE_ACCOUNT_AIRDROP_ACCOUNT;
    #[test]
    fn test_pack_distribute_token() {
        let user_portfolio_account = Pubkey::new_unique();
        let amount = 12;
        let nft_find = 0;
        let state_user_distribute = 0;

        let _user_distribute = UserDistributeStruct {
            user_portfolio_account,
            amount,
            nft_find,
            state_user_distribute,
        };
        let mut list_user_distribute = Vec::new();
        for _i in 0..NUMBER_USERS_DISTRIBUTE {
            let user_distribute = UserDistributeStruct {
                user_portfolio_account,
                amount,
                nft_find,
                state_user_distribute,
            };
            list_user_distribute.push(user_distribute);
        }

        let distribute_token = DistributeTokens {
            type_account: TYPE_ACCOUNT_AIRDROP_ACCOUNT,
            state_distribute_tokens: 0,
            portfolio_address: Pubkey::new_unique(),
            list_user_distribute,
        };

        const LEN: usize = DISTRIBUTE_TOKEN_SIZE;
        let mut packed = [0u8; LEN];
        PackDistributeTokens::pack_distribute_token(&distribute_token, &mut packed[..]);
        let unpacked = DistributeTokens::unpack_distribute_token(&packed).unwrap();
        assert_eq!(distribute_token, unpacked);
        assert_eq!(unpacked.type_account,TYPE_ACCOUNT_AIRDROP_ACCOUNT);
    }
}
