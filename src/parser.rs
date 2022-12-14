// input format:
// type(str), client(u16), tx(u32), amount(float)
// type -> (deposit, withdrawal, dispute, resolve, chargeback)

use std::error::Error;
use serde::Deserialize;
use csv;
use crate::common_types::{ClientID, TransactionID, Transaction, Amount};
use log::{warn, error};

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename(deserialize="type"))]
    transaction_type: String,
    client: ClientID,
    #[serde(rename(deserialize="tx"))]
    transaction: TransactionID,
    amount: Option<f64>
}

pub fn parse_csv<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let rows = reader.deserialize();

    Ok(rows.filter_map(|row: Result<Row, csv::Error>| {
        if row.is_err() {
            error!("Error while parsing the row: {}", row.err().unwrap());
            return None;
        }
        let row = row.unwrap();
        let amount = if let Some(amount) = row.amount {
            if amount < 0.0 {
                Some(0)
            } else {
                Some((amount * 10000.0) as Amount) // fixed floating point with 4 decimals
            }
        } else {
            None
        };
        match row.transaction_type.as_str() {
            "deposit" => {
                if let Some(amount) = amount {
                    Some(Transaction::Deposit(row.transaction, row.client, amount))
                } else {
                    warn!("No amount for transaction type 'deposit', skipping");
                    None
                }
            },
            "withdrawal" => {
                if let Some(amount) = amount {
                    Some(Transaction::Withdrawal(row.transaction, row.client, amount))
                } else {
                    warn!("No amount for transaction type 'withdrawal', skipping");
                    None
                }
            },
            "dispute" => Some(Transaction::Dispute(row.transaction, row.client)),
            "resolve" => Some(Transaction::Resolve(row.transaction, row.client)),
            "chargeback" => Some(Transaction::Chargeback(row.transaction, row.client)),
            _ => {
                warn!("Transaction type '{}' not supported, skipping", row.transaction_type);
                None
            }
        }
    }).collect())
}