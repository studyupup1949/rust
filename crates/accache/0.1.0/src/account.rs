use serde::{Deserialize, Serialize};
use solana_account::Account;
use solana_pubkey::Pubkey;

/// An [`Account`] paired with the [`Pubkey`] it was retrieved under.
///
/// Converts cleanly to `(Pubkey, Account)` for handoff to test harnesses such as
/// `mollusk-svm` and `litesvm`, both of which take account tuples directly.
///
/// ```ignore
/// use accache::KeyedAccount;
/// let keyed: KeyedAccount = /* ... */;
/// // For a single account:
/// let tuple = keyed.as_tuple();
/// // For a batch:
/// let accounts: Vec<(_, _)> = vec![keyed].into_iter().map(Into::into).collect();
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyedAccount {
    pub key: Pubkey,
    pub account: Account,
}

impl KeyedAccount {
    pub fn new(key: Pubkey, account: Account) -> Self {
        Self { key, account }
    }

    pub fn key(&self) -> &Pubkey {
        &self.key
    }

    pub fn account(&self) -> &Account {
        &self.account
    }

    /// Clone-and-pair; convenient when the `KeyedAccount` itself is borrowed.
    pub fn as_tuple(&self) -> (Pubkey, Account) {
        (self.key, self.account.clone())
    }

    pub fn into_tuple(self) -> (Pubkey, Account) {
        (self.key, self.account)
    }
}

impl From<KeyedAccount> for (Pubkey, Account) {
    fn from(value: KeyedAccount) -> Self {
        value.into_tuple()
    }
}

impl From<&KeyedAccount> for (Pubkey, Account) {
    fn from(value: &KeyedAccount) -> Self {
        value.as_tuple()
    }
}

impl From<(Pubkey, Account)> for KeyedAccount {
    fn from((key, account): (Pubkey, Account)) -> Self {
        Self { key, account }
    }
}
