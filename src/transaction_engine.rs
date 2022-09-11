use std::collections::HashMap;
use crate::common_types::*;

#[derive(Debug)]
pub struct Account {
    client_id: ClientID,
    amount: Amount,
    amount_held: Amount,
    is_locked: bool
}

impl Account {
    fn new(cx: ClientID, amount: Amount) -> Self {
        Self {
            client_id: cx,
            amount,
            amount_held: 0,
            is_locked: false
        }
    }
}

#[derive(Debug)]
pub enum Transaction {
    Deposit(TransactionID, ClientID, Amount)
}

#[derive(Debug)]
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
            Deposit(_, cx, amount) => self.process_deposit(cx, amount)
        }
    }

    pub fn get_accounts(&self) -> impl Iterator<Item=&Account> {
        self.accounts.iter().map(|(_, v)| v)
    }

    fn process_deposit(&mut self, cx: ClientID, amount: Amount) {
        if let Some(mut account) = self.accounts.get_mut(&cx) {
            if !account.is_locked {
                account.amount += amount;
            }
        } else {
            self.accounts.insert(cx, Account::new(cx, amount));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_engine::{Account, Transaction, TransactionEngine};

    #[test]
    fn create_account() {
        let account = Account::new(1, 42);
        assert_eq!(account.client_id, 1);
        assert_eq!(account.amount, 42);
        assert_eq!(account.amount_held, 0);
        assert_eq!(account.is_locked, false);
    }

    #[test]
    fn test_normal_deposit() {
        let mut te = TransactionEngine::new();
        let tx = Transaction::Deposit(1, 1, 42);
        te.process_transaction(tx);
        let accounts: Vec<&Account> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].amount, 42);
    }
}