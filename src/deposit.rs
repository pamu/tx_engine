use crate::errors::TxProcessingError;
use crate::processor::TxProcessor;
use crate::state::AppState;
use crate::tx::Tx;

#[derive(Debug, PartialEq, Clone)]
pub struct Deposit {
    pub client: u16,
    pub tx: u32,
    pub amount: f64,
}

impl Tx for Deposit {
    fn process(
        &self,
        state: &mut AppState,
        visitor: &dyn TxProcessor,
    ) -> Result<(), TxProcessingError> {
        visitor.process_deposit(state, self)
    }
}

#[cfg(test)]
mod tests {
    use super::Deposit;
    use crate::account::ClientAccount;
    use crate::errors::ErrCause;
    use crate::errors::TxProcessingError;
    use crate::processor::TxProcessorImpl;
    use crate::state::AppState;
    use crate::state::Flag;
    use crate::state::FlaggedDeposit;
    use crate::tx::Tx;

    #[test]
    fn deposit_creates_client_account_with_amount() {
        let deposit = Deposit {
            client: 1,
            tx: 2,
            amount: 200.1234,
        };
        let mut state = AppState::new();
        deposit.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.accounts.get(&deposit.client).unwrap(),
            &ClientAccount {
                client: 1,
                available: 200.1234,
                held: 0.0,
                total: 200.1234,
                locked: false,
            }
        )
    }

    #[test]
    fn deposit_increases_client_account_funds() {
        let deposit = Deposit {
            client: 1,
            tx: 2,
            amount: 200.1234,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: 1,
                available: 10.0,
                held: 0.0,
                total: 10.0,
                locked: false,
            },
        );
        deposit.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.accounts.get(&deposit.client).unwrap(),
            &ClientAccount {
                client: 1,
                available: 210.1234,
                held: 0.0,
                total: 210.1234,
                locked: false,
            }
        )
    }

    #[test]
    fn deposit_does_not_affect_held_funds() {
        let deposit = Deposit {
            client: 1,
            tx: 2,
            amount: 200.1234,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: 1,
                available: 10.0,
                held: 100.0,
                total: 10.0,
                locked: false,
            },
        );
        deposit.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.accounts.get(&deposit.client).unwrap(),
            &ClientAccount {
                client: 1,
                available: 210.1234,
                held: 100.0,
                total: 210.1234,
                locked: false,
            }
        )
    }

    #[test]
    fn deposit_gets_cached() {
        let deposit = Deposit {
            client: 1,
            tx: 2,
            amount: 200.1234,
        };
        let mut state = AppState::new();
        deposit.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.deposits.get(&deposit.tx).unwrap(),
            &FlaggedDeposit {
                deposit: Deposit {
                    client: 1,
                    tx: 2,
                    amount: 200.1234,
                },
                flag: Flag::NotDisputed
            }
        )
    }

    #[test]
    fn reject_deposit_with_negative_amount() {
        let negative_amount_deposit = Deposit {
            client: 1,
            tx: 2,
            amount: -10.0,
        };
        let mut state = AppState::new();
        let negative_deposit_processing_error = negative_amount_deposit
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();
        assert_eq!(
            negative_deposit_processing_error,
            TxProcessingError::Deposit(ErrCause::AmountLessThanOrEqualToZero)
        );
    }

    #[test]
    fn reject_deposit_with_zero_amount() {
        let zero_amount_deposit = Deposit {
            client: 1,
            tx: 2,
            amount: 0.0,
        };
        let mut state = AppState::new();
        let zero_deposit_processing_error = zero_amount_deposit
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();
        assert_eq!(
            zero_deposit_processing_error,
            TxProcessingError::Deposit(ErrCause::AmountLessThanOrEqualToZero)
        );
    }

    #[test]
    fn deposit_fails_if_client_account_is_locked() {
        let client = 1;
        let mut state = AppState::new();
        state.accounts.insert(
            client,
            ClientAccount {
                client: client,
                available: 100.0,
                held: 0.0,
                total: 100.0,
                locked: true,
            },
        );
        let deposit = Deposit {
            client: client,
            tx: 1,
            amount: 10.0,
        };
        let deposit_error = deposit.process(&mut state, &TxProcessorImpl).unwrap_err();

        assert_eq!(
            deposit_error,
            TxProcessingError::Deposit(ErrCause::ClientAccountLocked)
        );
    }
}
