use crate::constants::{
    account_size::{ADMIN_ACCOUNT_SIZE, NUMBER_OF_ADMINS, USER_ADMIN_SIZE, ADMIN_ACCOUNT_PREFIX_SIZE},
    constant::PUBKEY_SIZE,
    account_type::TYPE_ACCOUNT_ADMIN_ACCOUNT,
};
use anchor_lang::prelude::{ProgramError, Pubkey};
use arrayref::{array_mut_ref, array_ref, array_refs};
use serde::{Deserialize, Serialize};
use solana_program::{
    entrypoint_deprecated::ProgramResult,
    program_pack::{IsInitialized, Pack, Sealed},
};

const STATE_ADMIN_SIZE: usize = 1;
const TYPE_SIZE: usize = 1;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]

pub struct UserAdminStruct {
    ///The account of admin.
    pub user_admin_account: Pubkey,
    /// true if the use  is admin and false the use is deleted .
    pub state: u8,
}
impl UserAdminStruct {
    /// add new user admin
    pub fn add_admin(&mut self, user_admin_account: Pubkey) -> ProgramResult {
        self.user_admin_account = user_admin_account;
        self.state = 1;
        Ok(())
    }
    /// delete an admin from the list
    pub fn delete_admin(&mut self) -> ProgramResult {
        self.state = 0;
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]

pub struct AdminAccount {
    /// The type of account.
    pub type_account: u8,
    /// list_admins: : list of [`UserAdminStruct`].
    pub list_admins: Vec<UserAdminStruct>,
}

impl Sealed for AdminAccount {}

impl IsInitialized for AdminAccount {
    fn is_initialized(&self) -> bool {
        return self.type_account == TYPE_ACCOUNT_ADMIN_ACCOUNT;
    }
}

impl Pack for AdminAccount {
    const LEN: usize = ADMIN_ACCOUNT_SIZE;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, ADMIN_ACCOUNT_SIZE];
        let (type_account, list_admins) =
            array_refs![src, TYPE_SIZE, USER_ADMIN_SIZE * NUMBER_OF_ADMINS];

        let mut user_admin_vec: Vec<UserAdminStruct> = Vec::with_capacity(NUMBER_OF_ADMINS);
        let mut offset = 0;
        for _ in 0..NUMBER_OF_ADMINS {
            let user_data = array_ref![list_admins, offset, USER_ADMIN_SIZE];
            #[allow(clippy::ptr_offset_with_cast)]
            let (user_admin_account, state) = array_refs![user_data, PUBKEY_SIZE, STATE_ADMIN_SIZE];
            user_admin_vec.push(UserAdminStruct {
                user_admin_account: Pubkey::new_from_array(*user_admin_account),
                state: u8::from_le_bytes(*state),
            });
            offset += USER_ADMIN_SIZE;
        }

        Ok(AdminAccount {
            type_account: u8::from_le_bytes(*type_account),
            list_admins: user_admin_vec.to_vec(),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, ADMIN_ACCOUNT_SIZE];
        let AdminAccount {
            type_account,
            list_admins,
        } = self;

        let mut buffer = [0; ADMIN_ACCOUNT_SIZE];
        buffer[0] = *type_account;

        let admin_vec_tmp = bincode::serialize(&list_admins).unwrap();
        let mut admin_data_tmp = [0; USER_ADMIN_SIZE * NUMBER_OF_ADMINS]; // maximum 10 admins
        admin_data_tmp[0..USER_ADMIN_SIZE * NUMBER_OF_ADMINS as usize]
            .clone_from_slice(&admin_vec_tmp[8..(USER_ADMIN_SIZE * NUMBER_OF_ADMINS) + 8]); // 0..8 : 8 bytes contain length of struct
        buffer[ADMIN_ACCOUNT_PREFIX_SIZE..ADMIN_ACCOUNT_SIZE]
            .clone_from_slice(&admin_data_tmp[0..USER_ADMIN_SIZE * NUMBER_OF_ADMINS as usize]);
        dst[0..ADMIN_ACCOUNT_SIZE].copy_from_slice(&buffer[0..ADMIN_ACCOUNT_SIZE]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::account_type::TYPE_ACCOUNT_ADMIN_ACCOUNT;
    #[test]
    fn test_pack_admin() {
        let mut list_admins = Vec::new();
        for _i in 0..NUMBER_OF_ADMINS {
            let user_admin = UserAdminStruct {
                user_admin_account: Pubkey::new_unique(),
                state: 1,
            };

            list_admins.push(user_admin);
        }

        let admin_account = AdminAccount {
            type_account: TYPE_ACCOUNT_ADMIN_ACCOUNT,
            list_admins,
        };

        let mut packed = [0u8; AdminAccount::LEN];

        Pack::pack_into_slice(&admin_account, &mut packed[..]);
        let unpacked = AdminAccount::unpack_from_slice(&packed).unwrap();
        assert_eq!(admin_account, unpacked);
        assert_eq!(unpacked.type_account, TYPE_ACCOUNT_ADMIN_ACCOUNT);
    }
}
