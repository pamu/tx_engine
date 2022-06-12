use crate::errors::TxProcessingError;
use crate::processor::TxProcessor;
use crate::state::AppState;
use crate::tx::Tx;

#[derive(Debug)]
pub struct Dispute {
    pub client: u16,
    pub tx: u32,
}

impl Tx for Dispute {
    fn process(
        &self,
        state: &mut AppState,
        visitor: &dyn TxProcessor,
    ) -> Result<(), TxProcessingError> {
        visitor.process_dispute(state, self)
    }
}

#[cfg(test)]
mod tests {
    use super::Dispute;
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
    fn dispute_fails_on_non_existent_deposit() {
        let dispute = Dispute { client: 1, tx: 2 };
        let mut state = AppState::new();
        let dispute_processing_error = dispute.process(&mut state, &TxProcessorImpl).unwrap_err();

        assert_eq!(
            dispute_processing_error,
            TxProcessingError::Dispute(ErrCause::ClientTxNotFound)
        );
    }

    #[test]
    fn dispute_fails_if_tx_is_already_in_dispute() {
        let client_id = 1;
        let tx_id = 2;
        let dispute = Dispute {
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
                flag: Flag::Disputed,
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

        let dispute_err = dispute.process(&mut state, &TxProcessorImpl).unwrap_err();
        assert_eq!(
            dispute_err,
            TxProcessingError::Dispute(ErrCause::ClientTxAlreadyInDispute)
        )
    }

    #[test]
    fn dispute_fails_if_client_in_does_not_match() {
        let dispute = Dispute { client: 1, tx: 1 };
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

        let dispute_err = dispute.process(&mut state, &TxProcessorImpl).unwrap_err();
        assert_eq!(
            dispute_err,
            TxProcessingError::Dispute(ErrCause::ClientDidNotMatch)
        )
    }

    #[test]
    #[should_panic]
    fn dispute_panics_if_account_is_absent_and_diposit_is_present() {
        let dispute = Dispute { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::NotDisputed,
            },
        );

        dispute.process(&mut state, &TxProcessorImpl).unwrap_err();
    }

    #[test]
    fn dispute_moves_the_deposit_amount_to_held() {
        let dispute = Dispute { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::NotDisputed,
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

        dispute.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(state.accounts.get(&1).unwrap().held, 20.0);
    }

    #[test]
    fn dispute_marks_the_deposit_disputed() {
        let dispute = Dispute { client: 1, tx: 1 };
        let mut state = AppState::new();
        state.deposits.insert(
            1,
            FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 1,
                    amount: 20.0,
                },
                flag: Flag::NotDisputed,
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

        dispute.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(state.deposits.get(&1).unwrap().flag, Flag::Disputed);
    }
}
