use crate::chargeback::Chargeback;
use crate::deposit::Deposit;
use crate::dispute::Dispute;
use crate::resolve::Resolve;
use crate::tx::Tx;
use crate::withdrawal::Withdrawal;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "type")]
    record_type: String,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

impl Record {
    pub fn to_tx(&self) -> Result<Box<dyn Tx>, Box<dyn Error>> {
        match (self.record_type.as_str(), self.amount) {
            ("deposit", Some(amount)) => {
                let desposit: Box<dyn Tx> = Box::new(Deposit {
                    client: self.client,
                    tx: self.tx,
                    amount,
                });
                Ok(desposit)
            }
            ("withdrawal", Some(amount)) => {
                let withdraw: Box<dyn Tx> = Box::new(Withdrawal {
                    client: self.client,
                    tx: self.tx,
                    amount,
                });
                Ok(withdraw)
            }
            ("dispute", None) => {
                let dispute: Box<dyn Tx> = Box::new(Dispute {
                    client: self.client,
                    tx: self.tx,
                });
                Ok(dispute)
            }
            ("resolve", None) => {
                let resolve: Box<dyn Tx> = Box::new(Resolve {
                    client: self.client,
                    tx: self.tx,
                });
                Ok(resolve)
            }
            ("chargeback", None) => {
                let chargeback: Box<dyn Tx> = Box::new(Chargeback {
                    client: self.client,
                    tx: self.tx,
                });
                Ok(chargeback)
            }
            (record_type, _) => Err(Box::<dyn Error>::from(format!(
                "Invalid csv row with type: {}",
                record_type
            ))),
        }
    }
}
