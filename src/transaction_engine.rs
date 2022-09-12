use std::collections::{HashMap, HashSet};
use crate::common_types::*;
use crate::account::Account;


pub struct TransactionEngine {
    accounts: HashMap<ClientID, Account>,
    transactions: HashSet<Transaction>
}

pub struct ClientAccount<'a> {
    pub client_id: ClientID,
    pub account: &'a Account
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashSet::new()
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) {
        use Transaction::*;
        let should_save_tx = match transaction {
            Deposit(_, cx, amount) => self.process_deposit(cx, amount),
            Withdrawal(_, cx, amount) => self.process_withdrawal(cx, amount),
            Dispute(tx, cx) => {
                self.process_dispute(tx, cx);
                false
            },
            Resolve(tx, cx) => {
                self.process_resolve(tx, cx);
                false
            },
            Chargeback(tx, cx) => {
                self.process_chargeback(tx, cx);
                false
            },
            Any(_) => panic!("Transaction::Any is not a valid transaction") // this is a logic error, should be logged / pushed back up
        };
        if should_save_tx {
            self.transactions.insert(transaction);
        }
    }

    pub fn get_accounts(&self) -> impl Iterator<Item=ClientAccount> {
        self.accounts.iter().map(|(k, v)| ClientAccount{client_id: *k, account: v})
    }

    fn process_deposit(&mut self, cx: ClientID, amount: Amount) -> bool {
        if let Some(account) = self.accounts.get_mut(&cx) {
            account.deposit(amount)
        } else {
            self.accounts.insert(cx, Account::new(amount));
            true
        }
    }

    fn process_withdrawal(&mut self, cx: ClientID, amount: Amount) -> bool {
        if let Some(account) = self.accounts.get_mut(&cx) {
            account.withdraw(amount)
        } else {
            false
        }
    }

    fn process_dispute(&mut self, tx: TransactionID, cx: ClientID) {
        if let Some(transaction) = self.transactions.get(&Transaction::Any(tx)) {
            if transaction.get_client_id() != cx {
                // wrong client ID
            }
            if let Some(account) = self.accounts.get_mut(&cx) {
                if let Some(amount) = transaction.get_amount() {
                    account.dispute(amount);
                }
            }
        }
    }

    fn process_resolve(&mut self, tx: TransactionID, cx: ClientID) {
        todo!()
    }

    fn process_chargeback(&mut self, tx: TransactionID, cx: ClientID) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_engine::{ClientAccount, Transaction, TransactionEngine};

    #[test]
    fn test_deposit_no_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_deposit_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Deposit(2, 1, 42));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].account.available(), 84);
    }

    #[test]
    fn test_withdrawal_no_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Withdrawal(1, 1, 42));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 0);
    }

    #[test]
    fn test_withdrawal_account() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Withdrawal(2, 1, 30));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.total(), 12);
    }

    #[test]
    fn test_dispute_normal() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Dispute(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 42);
        assert_eq!(accounts[0].account.available(), 0);
    }

    #[test]
    fn test_dispute_partial() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 12);
        assert_eq!(accounts[0].account.available(), 30);
    }

    #[test]
    fn test_dispute_wrong_tx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(3, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_dispute_wrong_cx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(1, 2));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }
}