use crate::account::ClientAccount;
use crate::chargeback::Chargeback;
use crate::deposit::Deposit;
use crate::dispute::Dispute;
use crate::errors::ErrCause;
use crate::errors::TxProcessingError;
use crate::resolve::Resolve;
use crate::state::AppState;
use crate::state::Flag;
use crate::state::FlaggedDeposit;
use crate::withdrawal::Withdrawal;

pub trait TxProcessor {
    fn process_deposit(
        &self,
        state: &mut AppState,
        deposit: &Deposit,
    ) -> Result<(), TxProcessingError>;
    fn process_withdrawal(
        &self,
        state: &mut AppState,
        withdrawal: &Withdrawal,
    ) -> Result<(), TxProcessingError>;
    fn process_dispute(
        &self,
        state: &mut AppState,
        dispute: &Dispute,
    ) -> Result<(), TxProcessingError>;
    fn process_resolve(
        &self,
        state: &mut AppState,
        resolve: &Resolve,
    ) -> Result<(), TxProcessingError>;
    fn process_chargeback(
        &self,
        state: &mut AppState,
        chargeback: &Chargeback,
    ) -> Result<(), TxProcessingError>;
}

pub struct TxProcessorImpl;

impl TxProcessorImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TxProcessorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TxProcessor for TxProcessorImpl {
    fn process_deposit(
        &self,
        state: &mut AppState,
        deposit: &Deposit,
    ) -> Result<(), TxProcessingError> {
        if deposit.amount <= 0.0 {
            return Err(TxProcessingError::Deposit(
                ErrCause::AmountLessThanOrEqualToZero,
            ));
        }

        let deposit_result = match state.get_account_as_mut(deposit.client) {
            Some(client_acccount) if client_acccount.locked => {
                Err(TxProcessingError::Deposit(ErrCause::ClientAccountLocked))
            }
            Some(client_account) => {
                client_account.deposit(deposit.amount);
                Ok(())
            }
            None => {
                state.accounts.insert(
                    deposit.client,
                    ClientAccount {
                        client: deposit.client,
                        available: deposit.amount,
                        held: 0.0,
                        total: deposit.amount,
                        locked: false,
                    },
                );
                Ok(())
            }
        };

        match deposit_result {
            ok @ Ok(()) => {
                state.deposits.insert(
                    deposit.tx,
                    FlaggedDeposit {
                        deposit: deposit.clone(),
                        flag: Flag::NotDisputed,
                    },
                );
                ok
            }
            err => err,
        }
    }

    fn process_withdrawal(
        &self,
        state: &mut AppState,
        withdrawal: &Withdrawal,
    ) -> Result<(), TxProcessingError> {
        if withdrawal.amount <= 0.0 {
            return Err(TxProcessingError::Withdrawal(
                ErrCause::AmountLessThanOrEqualToZero,
            ));
        }

        match state.accounts.get_mut(&withdrawal.client) {
            Some(client_account) if client_account.locked => {
                Err(TxProcessingError::Withdrawal(ErrCause::ClientAccountLocked))
            }
            Some(client_account) if client_account.available < withdrawal.amount => {
                Err(TxProcessingError::Withdrawal(ErrCause::InsufficientFunds))
            }
            Some(client_account) => {
                client_account.withdraw(withdrawal.amount);
                Ok(())
            }
            None => Err(TxProcessingError::Withdrawal(
                ErrCause::ClientAccountNotFound,
            )),
        }
    }

    fn process_dispute(
        &self,
        state: &mut AppState,
        dispute: &Dispute,
    ) -> Result<(), TxProcessingError> {
        let curr_tx = dispute.tx;
        let flagged_deposit = match state.deposits.get(&curr_tx).cloned() {
            None => return Err(TxProcessingError::Dispute(ErrCause::ClientTxNotFound)),
            Some(deposit) => deposit,
        };

        if flagged_deposit.deposit.client != dispute.client {
            return Err(TxProcessingError::Dispute(ErrCause::ClientDidNotMatch));
        }

        if flagged_deposit.is_disputed() {
            return Err(TxProcessingError::Dispute(
                ErrCause::ClientTxAlreadyInDispute,
            ));
        }

        match state.get_account_as_mut(dispute.client) {
            Some(client_account) if client_account.locked => {
                Err(TxProcessingError::Dispute(ErrCause::ClientAccountLocked))
            }
            Some(client_account) => {
                client_account.held += flagged_deposit.deposit.amount;
                client_account.available -= flagged_deposit.deposit.amount;
                if let Some(deposit) = state.get_tx_as_mut(curr_tx) {
                    deposit.mark_disputed();
                }
                Ok(())
            }
            None => panic!("Account should exist for client in {:?}", dispute),
        }
    }

    fn process_resolve(
        &self,
        state: &mut AppState,
        resolve: &Resolve,
    ) -> Result<(), TxProcessingError> {
        let curr_tx = resolve.tx;
        let flagged_deposit = match state.deposits.get(&curr_tx).cloned() {
            None => return Err(TxProcessingError::Resolve(ErrCause::ClientTxNotFound)),
            Some(deposit) => deposit,
        };

        if flagged_deposit.deposit.client != resolve.client {
            return Err(TxProcessingError::Resolve(ErrCause::ClientDidNotMatch));
        }

        if !flagged_deposit.is_disputed() {
            return Err(TxProcessingError::Resolve(
                ErrCause::ClientTxIsNotUnderDispute,
            ));
        }

        let deposit = flagged_deposit.deposit;

        match state.get_account_as_mut(resolve.client) {
            Some(client_account) if client_account.locked => {
                Err(TxProcessingError::Resolve(ErrCause::ClientAccountLocked))
            }
            Some(client_account) => {
                client_account.held -= deposit.amount;
                client_account.available += deposit.amount;
                if let Some(deposit) = state.get_tx_as_mut(curr_tx) {
                    deposit.mark_resolved();
                }
                Ok(())
            }
            None => panic!("Account should exist for client in {:?}", resolve),
        }
    }

    fn process_chargeback(
        &self,
        state: &mut AppState,
        chargeback: &Chargeback,
    ) -> Result<(), TxProcessingError> {
        let curr_tx = chargeback.tx;
        let flagged_deposit = match state.get_tx(curr_tx).cloned() {
            None => return Err(TxProcessingError::Chargeback(ErrCause::ClientTxNotFound)),
            Some(deposit) => deposit,
        };

        if flagged_deposit.deposit.client != chargeback.client {
            return Err(TxProcessingError::Chargeback(ErrCause::ClientDidNotMatch));
        }

        if !flagged_deposit.is_disputed() {
            return Err(TxProcessingError::Chargeback(
                ErrCause::ClientTxIsNotUnderDispute,
            ));
        }

        let deposit = flagged_deposit.deposit;

        match state.get_account_as_mut(chargeback.client) {
            Some(client_account) if client_account.locked => {
                Err(TxProcessingError::Chargeback(ErrCause::ClientAccountLocked))
            }
            Some(ref mut client_account) => {
                client_account.held -= deposit.amount;
                client_account.total -= deposit.amount;
                client_account.locked = true;
                if let Some(deposit) = state.get_tx_as_mut(curr_tx) {
                    deposit.mark_chargedback();
                }
                Ok(())
            }
            None => panic!("Account should exist for client in {:?}", chargeback),
        }
    }
}
