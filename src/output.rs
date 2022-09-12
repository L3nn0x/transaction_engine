use crate::transaction_engine::ClientAccount;

pub fn output_accounts<'a>(accounts: impl Iterator<Item=ClientAccount<'a>>) {
    println!("client,available,held,total,locked");
    for account in accounts {
        output_account(account);
    }
}

fn output_account(account: ClientAccount) {
    println!("{},{},{},{},{}", account.client_id,
             account.account.available() as f64 / 10000.0,
             account.account.held() as f64 / 10000.0,
             account.account.total() as f64 / 10000.0,
             account.account.is_locked());
}