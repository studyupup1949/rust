//! Account-based state model (Ethereum-style).

use std::collections::HashMap;

/// An account in the state.
#[derive(Debug, Clone)]
pub struct Account {
    pub address: [u8; 20],
    pub nonce: u64,
    pub balance: u64,
    pub code_hash: Option<[u8; 32]>,
    pub storage_root: Option<[u8; 32]>,
}

impl Account {
    pub fn new(address: [u8; 20]) -> Self {
        Self { address, nonce: 0, balance: 0, code_hash: None, storage_root: None }
    }

    pub fn with_balance(mut self, balance: u64) -> Self { self.balance = balance; self }
    pub fn with_code(mut self, hash: [u8; 32]) -> Self { self.code_hash = Some(hash); self }
}

/// A state transition (transaction execution).
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: [u8; 20],
    pub to: Option<[u8; 20]>, // None = contract creation
    pub value: u64,
    pub nonce: u64,
    pub gas_used: u64,
}

/// Result of applying a state transition.
#[derive(Debug)]
pub struct ApplyResult {
    pub gas_used: u64,
    pub logs: Vec<Log>,
    pub created: Option<[u8; 20]>,
}

/// An event log.
#[derive(Debug, Clone)]
pub struct Log {
    pub address: [u8; 20],
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
}

/// World state: mapping of addresses to accounts.
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    accounts: HashMap<[u8; 20], Account>,
}

impl WorldState {
    pub fn new() -> Self { Self::default() }

    /// Get an account.
    pub fn get_account(&self, addr: &[u8; 20]) -> Option<&Account> {
        self.accounts.get(addr)
    }

    /// Get a mutable account, creating if needed.
    pub fn get_or_create(&mut self, addr: [u8; 20]) -> &mut Account {
        self.accounts.entry(addr).or_insert_with(|| Account::new(addr))
    }

    /// Apply a value transfer.
    pub fn transfer(&mut self, st: &StateTransition) -> Result<ApplyResult, String> {
        let from = self.accounts.get(&st.from)
            .ok_or_else(|| "sender account not found".to_string())?
            .clone();

        if from.nonce != st.nonce {
            return Err(format!("nonce mismatch: expected {}, got {}", from.nonce, st.nonce));
        }
        if from.balance < st.value {
            return Err("insufficient balance".into());
        }

        // Debit sender
        self.accounts.get_mut(&st.from).unwrap().balance -= st.value;
        self.accounts.get_mut(&st.from).unwrap().nonce += 1;

        // Credit receiver
        if let Some(to) = st.to {
            let recv = self.get_or_create(to);
            recv.balance += st.value;
        }

        Ok(ApplyResult { gas_used: st.gas_used, logs: vec![], created: None })
    }

    /// Mint new tokens to an account.
    pub fn mint(&mut self, addr: [u8; 20], amount: u64) {
        self.get_or_create(addr).balance += amount;
    }

    /// Burn tokens from an account.
    pub fn burn(&mut self, addr: &[u8; 20], amount: u64) -> Result<(), String> {
        let acc = self.accounts.get_mut(addr)
            .ok_or("account not found")?;
        if acc.balance < amount { return Err("insufficient balance".into()); }
        acc.balance -= amount;
        Ok(())
    }

    pub fn account_count(&self) -> usize { self.accounts.len() }
    pub fn total_supply(&self) -> u64 {
        self.accounts.values().map(|a| a.balance).sum()
    }
}

/// State snapshot for rollback support.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    accounts: HashMap<[u8; 20], Account>,
}

impl WorldState {
    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot { accounts: self.accounts.clone() }
    }

    pub fn rollback(&mut self, snap: StateSnapshot) {
        self.accounts = snap.accounts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer() {
        let mut state = WorldState::new();
        let alice = [1u8; 20];
        let bob = [2u8; 20];
        state.mint(alice, 1000);

        let st = StateTransition {
            from: alice, to: Some(bob), value: 300, nonce: 0, gas_used: 21_000,
        };
        state.transfer(&st).unwrap();

        assert_eq!(state.get_account(&alice).unwrap().balance, 700);
        assert_eq!(state.get_account(&bob).unwrap().balance, 300);
    }

    #[test]
    fn test_snapshot_rollback() {
        let mut state = WorldState::new();
        let addr = [1u8; 20];
        state.mint(addr, 500);
        let snap = state.snapshot();
        state.burn(&addr, 500).unwrap();
        assert_eq!(state.get_account(&addr).unwrap().balance, 0);
        state.rollback(snap);
        assert_eq!(state.get_account(&addr).unwrap().balance, 500);
    }
}
