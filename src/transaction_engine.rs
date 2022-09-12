use std::collections::HashMap;
use crate::common_types::*;
use crate::account::Account;
use log::{warn, info};

struct InnerTransaction {
    client_id: ClientID,
    is_disputed: bool,
    amount: Amount
}

pub struct TransactionEngine {
    accounts: HashMap<ClientID, Account>,
    transactions: HashMap<TransactionID, InnerTransaction>
}

pub struct ClientAccount<'a> {
    pub client_id: ClientID,
    pub account: &'a Account
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new()
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) {
        info!("Processing {:?}", transaction);
        use Transaction::*;
        let tx_to_save = match transaction {
            Deposit(tx, cx, amount) => {
                if self.process_deposit(cx, amount) {
                    Some((tx, InnerTransaction{
                        client_id: cx,
                        is_disputed: false,
                        amount
                    }))
                } else {
                    None
                }
            },
            Withdrawal(_, cx, amount) => {
                self.process_withdrawal(cx, amount);
                None
            },
            Dispute(tx, cx) => {
                self.process_dispute(tx, cx);
                None
            },
            Resolve(tx, cx) => {
                self.process_resolve(tx, cx);
                None
            },
            Chargeback(tx, cx) => {
                self.process_chargeback(tx, cx);
                None
            }
        };
        if let Some((tx, transaction)) = tx_to_save {
            self.transactions.insert(tx, transaction);
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
            warn!("Withdrawal transaction type on non-existing account, skipping cx={}", cx);
            false
        }
    }

    fn process_dispute(&mut self, tx: TransactionID, cx: ClientID) {
        if let Some(transaction) = self.transactions.get_mut(&tx) {
            if transaction.client_id != cx || transaction.is_disputed {
                // wrong client ID or that transaction is already disputed
                warn!("Dispute transaction type on wrong account or wrong transaction, skipping cx={} tx={}", cx, tx);
                return;
            }
            if let Some(account) = self.accounts.get_mut(&cx) {
                account.dispute(transaction.amount);
                transaction.is_disputed = true;
            }
        }
    }

    fn process_resolve_or_chargeback<Func: FnOnce(&mut Account, Amount)>(&mut self, tx: TransactionID, cx: ClientID, func: Func) {
        if let Some(transaction) = self.transactions.get(&tx) {
            if transaction.client_id != cx || !transaction.is_disputed {
                // wrong client ID or that transaction is not disputed
                warn!("Resolve/Chargeback transaction type on wrong account or wrong transaction, skipping cx={} tx={}", cx, tx);
                return;
            }
            if let Some(account) = self.accounts.get_mut(&cx) {
                func(account, transaction.amount);
            }
            self.transactions.remove(&tx);
        }
    }

    fn process_resolve(&mut self, tx: TransactionID, cx: ClientID) {
        self.process_resolve_or_chargeback(tx, cx, |account, amount| {
            account.resolve(amount);
        });
    }

    fn process_chargeback(&mut self, tx: TransactionID, cx: ClientID) {
        self.process_resolve_or_chargeback(tx, cx, |account, amount| {
            account.chargeback(amount);
        });
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
    fn test_dispute_twice() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(2, 1));
        te.process_transaction(Transaction::Dispute(2, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 30);
        assert_eq!(accounts[0].account.available(), 12);
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

    #[test]
    fn test_resolve_normal() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Dispute(1, 1));
        te.process_transaction(Transaction::Resolve(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_resolve_partial() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(1, 1));
        te.process_transaction(Transaction::Resolve(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_resolve_wrong_tx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Resolve(3, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_resolve_wrong_cx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Resolve(1, 2));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_resolve_tx_not_under_dispute() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Resolve(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_resolve_twice() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(1, 1));
        te.process_transaction(Transaction::Resolve(1, 1));
        te.process_transaction(Transaction::Resolve(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_chargeback_normal() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 42));
        te.process_transaction(Transaction::Dispute(1, 1));
        te.process_transaction(Transaction::Chargeback(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 0);
        assert_eq!(accounts[0].account.is_locked(), true);
    }

    #[test]
    fn test_chargeback_partial() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Dispute(1, 1));
        te.process_transaction(Transaction::Chargeback(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 30);
    }

    #[test]
    fn test_chargeback_wrong_tx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Chargeback(3, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_chargeback_wrong_cx() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Chargeback(1, 2));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }

    #[test]
    fn test_chargeback_tx_not_under_dispute() {
        let mut te = TransactionEngine::new();
        te.process_transaction(Transaction::Deposit(1, 1, 12));
        te.process_transaction(Transaction::Deposit(2, 1, 30));
        te.process_transaction(Transaction::Chargeback(1, 1));
        let accounts: Vec<ClientAccount> = te.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account.held(), 0);
        assert_eq!(accounts[0].account.available(), 42);
    }
}