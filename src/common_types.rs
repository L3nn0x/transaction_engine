use std::hash::Hasher;

pub type TransactionID = u32;
pub type ClientID = u16;
pub type Amount = u64;

#[derive(Debug, Eq, Clone, Copy)]
pub enum Transaction {
    Deposit(TransactionID, ClientID, Amount),
    Withdrawal(TransactionID, ClientID, Amount),
    Dispute(TransactionID, ClientID),
    Resolve(TransactionID, ClientID),
    Chargeback(TransactionID, ClientID)
}

impl Transaction {
    fn get_transaction_id(&self) -> TransactionID {
        use Transaction::*;
        match *self {
            Deposit(tx, _, _) => tx,
            Withdrawal(tx, _, _) => tx,
            Dispute(tx, _) => tx,
            Resolve(tx, _) => tx,
            Chargeback(tx, _) => tx
        }
    }
}

impl std::hash::Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let tx = self.get_transaction_id();
        tx.hash(state);
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        let self_tx = self.get_transaction_id();
        let other_tx = other.get_transaction_id();
        self_tx == other_tx
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use crate::common_types::Transaction;
    use std::hash::{Hash, Hasher};
    #[test]
    fn hash_test() {
        let tx1 = Transaction::Deposit(1, 1, 1);
        let tx2 = Transaction::Dispute(1, 1);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        tx1.hash(&mut hasher1);
        tx2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn eq_test() {
        let tx1 = Transaction::Deposit(1, 1, 1);
        let tx2 = Transaction::Dispute(1, 1);

        assert_eq!(tx1, tx2);
    }
}