//! State transition types
use crate::constants::{constant::PUBKEY_SIZE, account_type::TYPE_ACCOUNT_PORTFOLIO_ACCOUNT, account_size::{PORTFOLIO_PREFIX, ASSET_LEN}};
use arrayref::{array_ref, array_refs};
use serde::{Deserialize, Serialize};
use solana_program::{
    entrypoint_deprecated::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};
use std::vec::Vec;

const METADATAHASH_SIZE: usize = 16;
const METADATAURL_SIZE: usize = 128;
const TYPE_SIZE: usize = 1;
const IS_INITIALIZED: usize = 1;
const ASSET_DATA_LEN_SIZE: usize = 1;
const NFT_TOKEN_SIZE: usize = 1;
const VERSION_SIZE: usize = 1;
const AMOUNT_SIZE: usize = 1;
const PERIODE_SIZE: usize = 1;
const PERCENTAGE_SIZE: usize = 1;

/// struct of asset of ppm
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetStruct {
    /// Amout of asset address.
    pub amount: u8,
    /// The address of asset.
    pub address_asset: Pubkey,
    /// Periode for this asset.
    pub periode: u8,
    /// Asset to sold into asset.
    pub asset_to_sold_into_asset: Pubkey,
    /// Percentage of asset.
    pub percentage: u8,
}

impl AssetStruct {
    /// Add new asset to portfolio
    pub fn add_new_asset(
        &mut self,
        amount: u8,
        address_asset: Pubkey,
        periode: u8,
        asset_to_sold_into_asset: Pubkey,
        percentage: u8,
    ) -> ProgramResult {
        self.amount = amount;
        self.address_asset = address_asset;
        self.periode = periode;
        self.asset_to_sold_into_asset = asset_to_sold_into_asset;
        self.percentage = percentage;

        Ok(())
    }
}

/// Portfolio data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Portfolio {
    /// The type of account.
    pub type_account: u8,
    /// The creator of the portfolio was the only who can make changes on the portfolio account.
    pub creator_portfolio: Pubkey,
    /// Save the data of the new portfolio.
    pub meta_data_url: Vec<u8>,
    /// Hash of dataUrl to insure the immuability of data.
    pub meta_data_hash: Vec<u8>,
    /// Is initialized portfolio.
    pub is_initialized: u8,
    /// Length of assets by portfolio.
    pub asset_data_len: u8,
    /// Nft token stake.
    pub nft_token: u8,
    /// version actual for portfolio struct.
    pub version: u8,
    /// The address of the splm_stake.
    pub splm_stake: Pubkey,
    /// The address of the extended_data.
    pub extended_data: Pubkey,
    ///  Assets informations.
    pub asset_data: Vec<AssetStruct>,
}

impl Sealed for Portfolio {}
/// check if portfolio is initialized
impl IsInitialized for Portfolio {
    fn is_initialized(&self) -> bool {
        return self.type_account == TYPE_ACCOUNT_PORTFOLIO_ACCOUNT;
    }
}

/// trait for pack and unpack portfolio
pub trait PackPortfolio {
    /// unpack portfolio
    fn unpack_portfolio(src: &[u8]) -> Result<Portfolio, ProgramError>;
    /// pack portfolio
    fn pack_portfolio(&self, dst: &mut [u8]);
}

// amount, address_asset, periode, asset_to_sold_into_asset
impl PackPortfolio for Portfolio {
    fn unpack_portfolio(src: &[u8]) -> Result<Self, ProgramError> {
        let len_asset_nbre = (&src.len() - PORTFOLIO_PREFIX) / ASSET_LEN;

        let src_fix = array_ref![&src, 0, PORTFOLIO_PREFIX];
        let (
            type_account,
            creator_portfolio,
            meta_data_url,
            meta_data_hash,
            is_initialized,
            asset_data_len,
            nft_token,
            version,
            splm_stake,
            extended_data,
        ) = array_refs![
            src_fix,
            TYPE_SIZE,
            PUBKEY_SIZE,
            METADATAURL_SIZE,
            METADATAHASH_SIZE,
            IS_INITIALIZED,
            ASSET_DATA_LEN_SIZE,
            NFT_TOKEN_SIZE,
            VERSION_SIZE,
            PUBKEY_SIZE,
            PUBKEY_SIZE
        ];

        let len_data: usize = ASSET_LEN * len_asset_nbre;

        let list_asset_data = &src[PORTFOLIO_PREFIX..PORTFOLIO_PREFIX + (len_data) as usize];

        let mut asset_vec: Vec<AssetStruct> = Vec::with_capacity(len_asset_nbre);

        // extract the list of assets for portfolio
        // len_asset_nbre is mutable contain the numbre of assets by portfolio
        let mut offset = 0;
        for _ in 0..len_asset_nbre {
            let asset_data = array_ref![list_asset_data, offset, ASSET_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (amount, address_asset, periode, asset_to_sold_into_asset, percentage) = array_refs![
                asset_data,
                AMOUNT_SIZE,
                PUBKEY_SIZE,
                PERIODE_SIZE,
                PUBKEY_SIZE,
                PERCENTAGE_SIZE
            ];
            asset_vec.push(AssetStruct {
                amount: u8::from_le_bytes(*amount),
                address_asset: Pubkey::new_from_array(*address_asset),
                periode: u8::from_le_bytes(*periode),
                asset_to_sold_into_asset: Pubkey::new_from_array(*asset_to_sold_into_asset),
                percentage: u8::from_le_bytes(*percentage),
            });
            offset += ASSET_LEN;
        }

        return Ok(Portfolio {
            type_account: u8::from_le_bytes(*type_account),
            creator_portfolio: Pubkey::new(creator_portfolio),
            meta_data_url: meta_data_url.to_vec(),
            meta_data_hash: meta_data_hash.to_vec(),
            is_initialized: u8::from_le_bytes(*is_initialized),
            asset_data_len: u8::from_le_bytes(*asset_data_len),
            nft_token: u8::from_le_bytes(*nft_token),
            version: u8::from_le_bytes(*version),
            splm_stake: Pubkey::new(splm_stake),
            extended_data: Pubkey::new(extended_data),
            asset_data: asset_vec.to_vec(),
        });
    }

    fn pack_portfolio(&self, dst: &mut [u8]) {
        let Portfolio {
            ref creator_portfolio,
            meta_data_url,
            meta_data_hash,
            type_account,
            is_initialized,
            asset_data_len,
            nft_token,
            version,
            splm_stake,
            extended_data,
            asset_data,
        } = self;

        let asset_data_len_usize = *asset_data_len as usize;
        let len_data_asset = ASSET_LEN * asset_data_len_usize;

        let mut buffer = [0; PORTFOLIO_PREFIX + ASSET_LEN * 10];

        buffer[0] = *type_account;
        let creator_portfolio_range = TYPE_SIZE..TYPE_SIZE + PUBKEY_SIZE;
        buffer[creator_portfolio_range].clone_from_slice(creator_portfolio.as_ref());
        let meta_data_url_range =
            TYPE_SIZE + PUBKEY_SIZE..TYPE_SIZE + PUBKEY_SIZE + METADATAURL_SIZE;
        buffer[meta_data_url_range].clone_from_slice(meta_data_url);

        let meta_data_hash_range = TYPE_SIZE + PUBKEY_SIZE + METADATAURL_SIZE
            ..TYPE_SIZE + PUBKEY_SIZE + METADATAURL_SIZE + METADATAHASH_SIZE;
        buffer[meta_data_hash_range].clone_from_slice(&meta_data_hash);

        let is_initialized_range = TYPE_SIZE + PUBKEY_SIZE + METADATAURL_SIZE + METADATAHASH_SIZE;
        buffer[is_initialized_range] = *is_initialized;
        let asset_data_len_range =
            TYPE_SIZE + PUBKEY_SIZE + METADATAURL_SIZE + METADATAHASH_SIZE + IS_INITIALIZED;
        buffer[asset_data_len_range] = *asset_data_len;

        let nft_token_range = TYPE_SIZE
            + PUBKEY_SIZE
            + METADATAURL_SIZE
            + METADATAHASH_SIZE
            + IS_INITIALIZED
            + ASSET_DATA_LEN_SIZE;
        buffer[nft_token_range] = *nft_token;
        let version_range = TYPE_SIZE
            + PUBKEY_SIZE
            + METADATAURL_SIZE
            + METADATAHASH_SIZE
            + IS_INITIALIZED
            + ASSET_DATA_LEN_SIZE
            + NFT_TOKEN_SIZE;
        buffer[version_range] = *version;
        let splm_stake_range = TYPE_SIZE
            + PUBKEY_SIZE
            + METADATAURL_SIZE
            + METADATAHASH_SIZE
            + IS_INITIALIZED
            + ASSET_DATA_LEN_SIZE
            + NFT_TOKEN_SIZE
            + VERSION_SIZE
            ..TYPE_SIZE
                + PUBKEY_SIZE
                + METADATAURL_SIZE
                + METADATAHASH_SIZE
                + IS_INITIALIZED
                + ASSET_DATA_LEN_SIZE
                + NFT_TOKEN_SIZE
                + VERSION_SIZE
                + PUBKEY_SIZE;
        buffer[splm_stake_range].clone_from_slice(splm_stake.as_ref());
        let extended_data_range = TYPE_SIZE
            + PUBKEY_SIZE
            + METADATAURL_SIZE
            + METADATAHASH_SIZE
            + IS_INITIALIZED
            + ASSET_DATA_LEN_SIZE
            + NFT_TOKEN_SIZE
            + VERSION_SIZE
            + PUBKEY_SIZE
            ..TYPE_SIZE
                + PUBKEY_SIZE
                + METADATAURL_SIZE
                + METADATAHASH_SIZE
                + IS_INITIALIZED
                + ASSET_DATA_LEN_SIZE
                + NFT_TOKEN_SIZE
                + VERSION_SIZE
                + PUBKEY_SIZE
                + PUBKEY_SIZE;
        buffer[extended_data_range].clone_from_slice(extended_data.as_ref());

        let asset_vec_tmp = bincode::serialize(&asset_data).unwrap();

        let mut asset_data_tmp = [0; ASSET_LEN * 10]; // MAXIMUM 10 ASSETS

        let len_asset = asset_vec_tmp.len();

        asset_data_tmp[0..len_data_asset as usize].clone_from_slice(&asset_vec_tmp[8..len_asset]); // 0..8 : 8 bytes contain length of struct

        buffer[PORTFOLIO_PREFIX..PORTFOLIO_PREFIX + len_data_asset as usize]
            .clone_from_slice(&asset_data_tmp[0..len_data_asset as usize]);

        let mut buffer_tranformed: Vec<u8> =
            Vec::with_capacity(PORTFOLIO_PREFIX + (len_data_asset as usize));
        buffer_tranformed.resize(PORTFOLIO_PREFIX + (len_data_asset as usize), 0);
        buffer_tranformed[0..PORTFOLIO_PREFIX + len_data_asset as usize]
            .copy_from_slice(&buffer[0..PORTFOLIO_PREFIX + (len_data_asset as usize)]);

        dst[0..PORTFOLIO_PREFIX + len_data_asset as usize].copy_from_slice(&buffer_tranformed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pack_asset_portfolio() {
        let amount = 12;
        let address_asset = Pubkey::new_unique();
        let periode = 2;
        let asset_to_sold_into_asset = Pubkey::new_unique();
        let percentage = 1;
        let number_assets: u8 = 1;
        let asset = AssetStruct {
            amount,
            address_asset,
            periode,
            asset_to_sold_into_asset,
            percentage,
        };
        let meta_data_url = [1; METADATAURL_SIZE];
        let meta_data_hash = [1; METADATAHASH_SIZE];
        let mut asset_data = Vec::new();
        asset_data.push(asset);
        let portfolio = Portfolio {
            type_account: TYPE_ACCOUNT_PORTFOLIO_ACCOUNT,
            creator_portfolio: Pubkey::new_unique(),
            meta_data_url: meta_data_url.to_vec(),
            meta_data_hash: meta_data_hash.to_vec(),
            is_initialized: 1,
            asset_data_len: number_assets,
            nft_token: 1,
            version: 2,
            splm_stake: Pubkey::new_unique(),
            extended_data: Pubkey::new_unique(),
            asset_data,
        };
        const LEN: usize = PORTFOLIO_PREFIX + (1 * ASSET_LEN);
        let mut packed = [0u8; LEN];
        PackPortfolio::pack_portfolio(&portfolio, &mut packed[..]);
        let unpacked = Portfolio::unpack_portfolio(&packed).unwrap();
        assert_eq!(portfolio, unpacked);
        assert_eq!(unpacked.type_account,TYPE_ACCOUNT_PORTFOLIO_ACCOUNT);
    }
}
