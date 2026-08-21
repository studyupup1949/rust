use crate::constants::{
    account_size::NFT_ACCOUNT_SIZE, constant::PUBKEY_SIZE, account_type::TYPE_ACCOUNT_NFT_ACCOUNT,
};
use anchor_lang::prelude::{ProgramError, Pubkey};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use serde::{Deserialize, Serialize};
use solana_program::program_pack::{IsInitialized, Pack, Sealed};

const TYPE_SIZE: usize = 1;
const STAKE_SIZE: usize = 1;
const NONCE_SIZE: usize = 1;

/// Account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NFTAccount {
    /// The type of account.
    pub type_account: u8,
    /// The owner of this account.
    pub owner: Pubkey,
    /// The  user stake his nft token.
    pub stake: u8,
    ///  The signer used by cross program .
    pub authority: Pubkey,
    /// Nonce used in program address for invoke a cross-program instruction..
    pub nonce: u8,
    /// Create account program for cross program.
    pub create_account_programm: Pubkey,
}

impl Sealed for NFTAccount {}

impl IsInitialized for NFTAccount {
    fn is_initialized(&self) -> bool {
        return self.type_account == TYPE_ACCOUNT_NFT_ACCOUNT;
    }
}

/// pack and unpack for nft account.
impl Pack for NFTAccount {
    const LEN: usize = NFT_ACCOUNT_SIZE;
    // unpack nft account
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, NFT_ACCOUNT_SIZE];
        let (type_account, owner, stake, authority, nonce, create_account_programm) = array_refs![
            src,
            TYPE_SIZE,
            PUBKEY_SIZE,
            STAKE_SIZE,
            PUBKEY_SIZE,
            NONCE_SIZE,
            PUBKEY_SIZE
        ];
        Ok(NFTAccount {
            type_account: u8::from_le_bytes(*type_account),
            owner: Pubkey::new_from_array(*owner),
            stake: u8::from_le_bytes(*stake),
            authority: Pubkey::new_from_array(*authority),
            nonce: u8::from_le_bytes(*nonce),
            create_account_programm: Pubkey::new_from_array(*create_account_programm),
        })
    }

    // pack nft account
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, NFT_ACCOUNT_SIZE];
        let (
            type_account_dst,
            owner_dst,
            stake_dst,
            authority_dst,
            nonce_dst,
            create_account_programm_dst,
        ) = mut_array_refs![
            dst,
            TYPE_SIZE,
            PUBKEY_SIZE,
            STAKE_SIZE,
            PUBKEY_SIZE,
            NONCE_SIZE,
            PUBKEY_SIZE
        ];
        let NFTAccount {
            type_account,
            owner,
            stake,
            authority,
            nonce,
            create_account_programm,
        } = self;
        *type_account_dst = type_account.to_le_bytes();
        owner_dst.copy_from_slice(owner.as_ref());
        *stake_dst = stake.to_le_bytes();
        authority_dst.copy_from_slice(authority.as_ref());
        *nonce_dst = nonce.to_le_bytes();
        create_account_programm_dst.copy_from_slice(create_account_programm.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::account_type::TYPE_ACCOUNT_NFT_ACCOUNT;

    #[test]
    fn test_pack_nft() {
        let type_account = TYPE_ACCOUNT_NFT_ACCOUNT;
        let owner = Pubkey::new_unique();
        let stake = 0;
        let authority = Pubkey::new_unique();
        let nonce = 255;
        let create_account_programm = Pubkey::new_unique();

        let nft = NFTAccount {
            type_account,
            owner,
            stake,
            authority,
            nonce,
            create_account_programm,
        };

        let mut packed = [0u8; NFTAccount::LEN];

        Pack::pack_into_slice(&nft, &mut packed[..]);
        let unpacked = NFTAccount::unpack_from_slice(&packed).unwrap();
        assert_eq!(nft, unpacked);
        assert_eq!(unpacked.type_account, TYPE_ACCOUNT_NFT_ACCOUNT);
    }
}
