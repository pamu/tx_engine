use crate::errors::TxProcessingError;
use crate::processor::TxProcessor;
use crate::state::AppState;
use std::fmt::Debug;

pub trait Tx: Debug {
    fn process(
        &self,
        state: &mut AppState,
        visitor: &dyn TxProcessor,
    ) -> Result<(), TxProcessingError>;
}
