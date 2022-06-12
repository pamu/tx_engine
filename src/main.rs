use crate::processor::TxProcessorImpl;
use crate::record::Record;
use crate::state::AppState;
use csv::Trim;
use std::env::args;
use std::error::Error;
use std::io;
use std::path::PathBuf;

pub mod account;
pub mod chargeback;
pub mod deposit;
pub mod dispute;
pub mod errors;
pub mod processor;
pub mod record;
pub mod resolve;
pub mod state;
pub mod tx;
pub mod withdrawal;

fn main() -> Result<(), Box<dyn Error>> {
    let tx_file =
        match args().nth(1) {
            None => return Err(Box::<dyn Error>::from(
                "Usage: tx_engine <transactions csv file> or cargo run -- <transactions csv file>",
            )),
            Some(file) => PathBuf::from(file),
        };
    process_tx_file(&tx_file)
}

fn process_tx_file(file: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(b',')
        .escape(None)
        .trim(Trim::All)
        .buffer_capacity(4098)
        .from_path(file)?;

    let mut state = AppState::new();

    for record in rdr.records() {
        let record: Record = record?.deserialize(None)?;
        let tx = record.to_tx()?;
        let _result = tx.process(&mut state, &TxProcessorImpl);
    }

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_writer(io::stdout());

    for v in state.accounts.values() {
        writer.serialize(v.writable_record())?;
    }

    writer.flush()?;
    Ok(())
}
