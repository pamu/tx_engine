use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum ErrCause {
    AmountLessThanOrEqualToZero,
    ClientAccountLocked,
    ClientAccountNotFound,
    InsufficientFunds,
    ClientTxNotFound,
    ClientTxAlreadyInDispute,
    ClientTxIsNotUnderDispute,
    ClientDidNotMatch,
}

#[derive(Debug, PartialEq)]
pub enum TxProcessingError {
    Deposit(ErrCause),
    Withdrawal(ErrCause),
    Dispute(ErrCause),
    Resolve(ErrCause),
    Chargeback(ErrCause),
}

impl Error for TxProcessingError {}

impl Display for TxProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxProcessingError::Deposit(cause) => write!(f, "{}", msg("Deposit", cause).as_str()),
            TxProcessingError::Withdrawal(cause) => {
                write!(f, "{}", msg("Withdrawal", cause).as_str())
            }
            TxProcessingError::Dispute(cause) => write!(f, "{}", msg("Dispute", cause).as_str()),
            TxProcessingError::Resolve(cause) => write!(f, "{}", msg("Resolve", cause).as_str()),
            TxProcessingError::Chargeback(cause) => {
                write!(f, "{}", msg("Chargeback", cause).as_str())
            }
        }
    }
}

fn msg(tag: &str, cause: &ErrCause) -> String {
    match cause {
        ErrCause::AmountLessThanOrEqualToZero => format!("{}: amount less than or equal to 0", tag),
        ErrCause::ClientAccountLocked => format!("{}: account locked", tag),
        ErrCause::ClientAccountNotFound => format!("{}: acount not found", tag),
        ErrCause::InsufficientFunds => format!("{}: insufficient funds", tag),
        ErrCause::ClientTxNotFound => format!("{}: transaction not found", tag),
        ErrCause::ClientTxAlreadyInDispute => format!("{}: transaction is already in dispute", tag),
        ErrCause::ClientTxIsNotUnderDispute => format!("{}: transaction is not under dispute", tag),
        ErrCause::ClientDidNotMatch => {
            format!(
                "{}: client in dispute/resolve/chargeback does not match client in deposit",
                tag
            )
        }
    }
}
