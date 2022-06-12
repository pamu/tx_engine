use serde::Serialize;

#[derive(Serialize)]
pub struct ClientAccountCsvWritableRecord {
    pub client: u16,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

#[derive(Debug, PartialEq)]
pub struct ClientAccount {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl ClientAccount {
    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw(&mut self, amount: f64) {
        self.available -= amount;
        self.total -= amount;
    }

    pub fn writable_record(&self) -> ClientAccountCsvWritableRecord {
        ClientAccountCsvWritableRecord {
            client: self.client,
            available: format!("{:.4}", self.available),
            held: format!("{:.4}", self.held),
            total: format!("{:.4}", self.total),
            locked: self.locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientAccount;

    #[test]
    fn deposit_increases_available_and_total_amounts() {
        let mut account = ClientAccount {
            client: 1,
            available: 10.0,
            held: 0.0,
            total: 10.0,
            locked: false,
        };

        account.deposit(10.0);

        assert_eq!(
            account,
            ClientAccount {
                client: 1,
                available: 20.0,
                held: 0.0,
                total: 20.0,
                locked: false
            }
        )
    }

    #[test]
    fn withdraw_decreases_available_and_total_amounts() {
        let mut account = ClientAccount {
            client: 1,
            available: 10.0,
            held: 0.0,
            total: 10.0,
            locked: false,
        };

        account.withdraw(10.0);

        assert_eq!(
            account,
            ClientAccount {
                client: 1,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                locked: false
            }
        )
    }
}
