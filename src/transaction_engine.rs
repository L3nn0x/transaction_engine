use std::collections::HashMap;
use crate::common_types::*;
use crate::account::Account;

#[derive(Debug)]
pub enum Transaction {
    Deposit(TransactionID, ClientID, Amount),
    Withdrawal(TransactionID, ClientID, Amount),
}

pub struct TransactionEngine {
    accounts: HashMap<ClientID, Account>
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new()
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) {
        use Transaction::*;
        match transaction {
            Deposit(_, cx, amount) => self.process_deposit(cx, amount),
            Withdrawal(_, cx, amount) => self.process_withdrawal(cx, amount)
        }
    }

    pub fn get_accounts(&self) -> impl Iterator<Item=&Account> {
        self.accounts.iter().map(|(_, v)| v)
    }

    fn process_deposit(&mut self, cx: ClientID, amount: Amount) {
        if let Some(account) = self.accounts.get_mut(&cx) {
            account.deposit(amount);
        } else {
            self.accounts.insert(cx, Account::new(cx, amount));
        }
    }

    fn process_withdrawal(&mut self, cx: ClientID, amount: Amount) {
        if let Some(account) = self.accounts.get_mut(&cx) {
            account.withdraw(amount);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_engine::{Account, Transaction, TransactionEngine};

    #[test]
    fn test_deposit_no_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        let accounts: Vec<&Account> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id(), 1);
        assert_eq!(accounts[0].available(), 42);
    }

    #[test]
    fn test_deposit_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Deposit(2, 1, 42));
        let accounts: Vec<&Account> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id(), 1);
        assert_eq!(accounts[0].available(), 84);
    }

    #[test]
    fn test_withdrawal_no_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Withdrawal(1, 1, 42));
        let accounts: Vec<&Account> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 0);
    }

    #[test]
    fn test_withdrawal_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Withdrawal(2, 1, 30));
        let accounts: Vec<&Account> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].total(), 12);
    }
}