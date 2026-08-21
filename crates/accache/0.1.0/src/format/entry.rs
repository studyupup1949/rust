use serde::{Deserialize, Serialize};
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::account::KeyedAccount;

/// On-disk record. Mirrors `solana_account::Account` plus a fetch timestamp so
/// `RefreshPolicy::Ttl` can be evaluated against persisted data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub key: [u8; 32],
    pub lamports: u64,
    pub owner: [u8; 32],
    pub executable: bool,
    pub rent_epoch: u64,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub fetched_at_unix_ms: Option<u64>,
}

impl Entry {
    pub fn from_keyed(keyed: &KeyedAccount, fetched_at_unix_ms: Option<u64>) -> Self {
        Self {
            key: keyed.key.to_bytes(),
            lamports: keyed.account.lamports,
            owner: keyed.account.owner.to_bytes(),
            executable: keyed.account.executable,
            rent_epoch: keyed.account.rent_epoch,
            data: keyed.account.data.clone(),
            fetched_at_unix_ms,
        }
    }

    pub fn into_keyed(self) -> KeyedAccount {
        KeyedAccount {
            key: Pubkey::new_from_array(self.key),
            account: Account {
                lamports: self.lamports,
                data: self.data,
                owner: Pubkey::new_from_array(self.owner),
                executable: self.executable,
                rent_epoch: self.rent_epoch,
            },
        }
    }

    pub fn pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.key)
    }
}

mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        // bincode emits `Vec<u8>` as a byte sequence; accept either form.
        Vec::<u8>::deserialize(d)
    }
}
