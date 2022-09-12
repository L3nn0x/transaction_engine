pub type TransactionID = u32;
pub type ClientID = u16;
pub type Amount = u64;

#[derive(Debug, Clone, Copy)]
pub enum Transaction {
    Deposit(TransactionID, ClientID, Amount),
    Withdrawal(TransactionID, ClientID, Amount),
    Dispute(TransactionID, ClientID),
    Resolve(TransactionID, ClientID),
    Chargeback(TransactionID, ClientID),
}