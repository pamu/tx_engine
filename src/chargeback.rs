use crate::errors::TxProcessingError;
use crate::processor::TxProcessor;
use crate::state::AppState;
use crate::tx::Tx;

#[derive(Debug)]
pub struct Chargeback {
    pub client: u16,
    pub tx: u32,
}

impl Tx for Chargeback {
    fn process(
        &self,
        state: &mut AppState,
        visitor: &dyn TxProcessor,
    ) -> Result<(), TxProcessingError> {
        visitor.process_chargeback(state, self)
    }
}

#[cfg(test)]
mod tests {
    use super::Chargeback;
    use crate::account::ClientAccount;
    use crate::deposit::Deposit;
    use crate::errors::ErrCause;
    use crate::errors::TxProcessingError;
    use crate::processor::TxProcessorImpl;
    use crate::state::AppState;
    use crate::state::Flag;
    use crate::state::FlaggedDeposit;
    use crate::tx::Tx;

    #[test]
    fn chargeback_fails_on_non_existent_deposit() {
        let chargeback = Chargeback { client: 1, tx: 2 };
        let mut state = AppState::new();
        let err = chargeback
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();

        assert_eq!(
            err,
            TxProcessingError::Chargeback(ErrCause::ClientTxNotFound)
        );
    }

    #[test]
    fn chargeback_fails_if_tx_not_in_dispute() {
        let client_id = 1;
        let tx_id = 2;
        let chargeback = Chargeback {
            client: client_id,
            tx: tx_id,
        };
        let mut state = AppState::new();
        state.deposits.insert(
            tx_id,
            FlaggedDeposit {
                deposit: Deposit {
                    client: client_id,
                    tx: tx_id,
                    amount: 20.0,
                },
                flag: Flag::NotDisputed,
            },
        );
        state.accounts.insert(
            client_id,
            ClientAccount {
                client: client_id,
                available: 10.0,
                held: 0.0,
                total: 10.0,
                locked: false,
            },
        );

        let err = chargeback
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();
        assert_eq!(
            err,
            TxProcessingError::Chargeback(ErrCause::ClientTxIsNotUnderDispute)
        )
    }

    #[test]
    fn dispute_fails_if_client_in_does_not_match() {
        let chargeback = Chargeback { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 2,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::NotDisputed,
            },
        );

        let err = chargeback
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();
        assert_eq!(
            err,
            TxProcessingError::Chargeback(ErrCause::ClientDidNotMatch)
        )
    }

    #[test]
    #[should_panic]
    fn chargeback_panics_if_account_is_absent_and_diposit_is_present() {
        let chargeback = Chargeback { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::Disputed,
            },
        );

        chargeback
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();
    }

    #[test]
    fn chargeback_locks_the_account() {
        let chargeback = Chargeback { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::Disputed,
            },
        );

        state.accounts.insert(
            1,
            ClientAccount {
                client: 1,
                available: 20.0,
                held: 0.0,
                total: 20.0,
                locked: false,
            },
        );

        chargeback.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(state.accounts.get(&1).unwrap().locked, true);
    }

    #[test]
    fn chargeback_marks_the_deposit_chargebacked() {
        let chargeback = Chargeback { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::Disputed,
            },
        );

        state.accounts.insert(
            1,
            ClientAccount {
                client: 1,
                available: 20.0,
                held: 0.0,
                total: 20.0,
                locked: false,
            },
        );

        chargeback.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(state.deposits.get(&1).unwrap().flag, Flag::Chargebacked);
    }
}
