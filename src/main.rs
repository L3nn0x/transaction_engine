use crate::transaction_engine::ClientAccount;

mod common_types;
mod transaction_engine;
mod account;
mod parser;

fn main() {
    let mut transaction_engine = transaction_engine::TransactionEngine::new();

    if let Ok(transactions) = parser::parse_csv("test_inputs/simple_input.csv") {
        for transaction in transactions {
            transaction_engine.process_transaction(transaction);
        }

        println!("client,available,held,total,locked");
        for account in transaction_engine.get_accounts() {
            output_account(&account);
        }
    } else {
        return;
    }
}

fn output_account(account: &ClientAccount) {
    println!("{},{},{},{},{}", account.client_id,
             account.account.available() as f64 / 10000.0,
             account.account.held() as f64 / 10000.0,
             account.account.total() as f64 / 10000.0,
             account.account.is_locked());
}