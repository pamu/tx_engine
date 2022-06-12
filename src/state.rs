use crate::account::ClientAccount;
use crate::deposit::Deposit;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Flag {
    NotDisputed,
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FlaggedDeposit {
    pub deposit: Deposit,
    pub flag: Flag,
}

impl FlaggedDeposit {
    pub fn is_disputed(&self) -> bool {
        self.flag == Flag::Disputed
    }

    pub fn mark_disputed(&mut self) {
        self.flag = Flag::Disputed
    }

    pub fn mark_resolved(&mut self) {
        self.flag = Flag::Resolved
    }

    pub fn mark_chargedback(&mut self) {
        self.flag = Flag::Chargebacked
    }
}

#[derive(Debug)]
pub struct AppState {
    pub accounts: HashMap<u16, ClientAccount>,
    pub deposits: HashMap<u32, FlaggedDeposit>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            deposits: HashMap::new(),
        }
    }

    pub fn get_account_as_mut(&mut self, client: u16) -> Option<&mut ClientAccount> {
        self.accounts.get_mut(&client)
    }

    pub fn get_tx_as_mut(&mut self, tx: u32) -> Option<&mut FlaggedDeposit> {
        self.deposits.get_mut(&tx)
    }

    pub fn get_tx(&self, tx: u32) -> Option<&FlaggedDeposit> {
        self.deposits.get(&tx)
    }
}
