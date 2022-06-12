use crate::errors::TxProcessingError;
use crate::processor::TxProcessor;
use crate::state::AppState;
use crate::tx::Tx;

#[derive(Debug)]
pub struct Withdrawal {
    pub client: u16,
    pub tx: u32,
    pub amount: f64,
}

impl Tx for Withdrawal {
    fn process(
        &self,
        state: &mut AppState,
        visitor: &dyn TxProcessor,
    ) -> Result<(), TxProcessingError> {
        visitor.process_withdrawal(state, self)
    }
}

#[cfg(test)]
mod tests {
    use super::Withdrawal;
    use crate::account::ClientAccount;
    use crate::errors::ErrCause;
    use crate::errors::TxProcessingError;
    use crate::processor::TxProcessorImpl;
    use crate::state::AppState;
    use crate::tx::Tx;

    #[test]
    fn withdraw_fails_if_amount_is_zero() {
        let client = 1;
        let zero_withdrawal = Withdrawal {
            client: client,
            tx: 2,
            amount: 0.0,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: client,
                available: 100.0,
                held: 0.0,
                total: 100.0,
                locked: false,
            },
        );
        let withdrawal_error = zero_withdrawal
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();

        assert_eq!(
            withdrawal_error,
            TxProcessingError::Withdrawal(ErrCause::AmountLessThanOrEqualToZero)
        );
    }

    #[test]
    fn withdraw_fails_if_amount_is_negative() {
        let client = 1;
        let negative_withdrawal = Withdrawal {
            client: client,
            tx: 2,
            amount: -100.0,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: client,
                available: 100.0,
                held: 0.0,
                total: 100.0,
                locked: false,
            },
        );
        let withdrawal_error = negative_withdrawal
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();

        assert_eq!(
            withdrawal_error,
            TxProcessingError::Withdrawal(ErrCause::AmountLessThanOrEqualToZero)
        );
    }

    #[test]
    fn withdraw_fails_when_balance_is_insufficient() {
        let client = 1;
        let withdrawal = Withdrawal {
            client: client,
            tx: 2,
            amount: 1000.0,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: client,
                available: 100.0,
                held: 0.0,
                total: 100.0,
                locked: false,
            },
        );
        let withdrawal_error = withdrawal
            .process(&mut state, &TxProcessorImpl)
            .unwrap_err();

        assert_eq!(
            withdrawal_error,
            TxProcessingError::Withdrawal(ErrCause::InsufficientFunds)
        );
    }

    #[test]
    fn withdraw_fails_if_client_account_is_locked() {
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
        let deposit = Withdrawal {
            client: client,
            tx: 1,
            amount: 10.0,
        };
        let withdrawal_error = deposit.process(&mut state, &TxProcessorImpl).unwrap_err();

        assert_eq!(
            withdrawal_error,
            TxProcessingError::Withdrawal(ErrCause::ClientAccountLocked)
        );
    }

    #[test]
    fn withdraw_does_not_affect_held_funds() {
        let withdrawal = Withdrawal {
            client: 1,
            tx: 2,
            amount: 10.0,
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
        withdrawal.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.accounts.get(&withdrawal.client).unwrap(),
            &ClientAccount {
                client: 1,
                available: 0.0,
                held: 100.0,
                total: 0.0,
                locked: false,
            }
        )
    }

    #[test]
    fn withdrawal_decrease_client_funds() {
        let withdrawal = Withdrawal {
            client: 1,
            tx: 2,
            amount: 10.0,
        };
        let mut state = AppState::new();
        state.accounts.insert(
            1,
            ClientAccount {
                client: 1,
                available: 100.0,
                held: 100.0,
                total: 100.0,
                locked: false,
            },
        );
        withdrawal.process(&mut state, &TxProcessorImpl).unwrap();

        assert_eq!(
            state.accounts.get(&withdrawal.client).unwrap(),
            &ClientAccount {
                client: 1,
                available: 90.0,
                held: 100.0,
                total: 90.0,
                locked: false,
            }
        )
    }
}
