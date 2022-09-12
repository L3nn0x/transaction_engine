use crate::common_types::*;

pub struct Account {
    client_id: ClientID,
    amount: Amount,
    amount_held: Amount,
    is_locked: bool
}

impl Account {
    pub fn new(cx: ClientID, amount: Amount) -> Self {
        Self {
            client_id: cx,
            amount,
            amount_held: 0,
            is_locked: false
        }
    }

    pub fn deposit(&mut self, amount: Amount) {
        if !self.is_locked {
            self.amount += amount;
        }
    }

    pub fn withdraw(&mut self, amount: Amount) {
        if amount <= self.available() && !self.is_locked() {
            self.amount -= amount;
        }
    }

    pub fn dispute(&mut self, amount: Amount) {
        if amount <= self.available() {
            self.amount_held += amount;
        }
    }

    pub fn client_id(&self) -> ClientID {
        self.client_id
    }

    pub fn available(&self) -> Amount {
        self.amount - self.amount_held
    }

    pub fn held(&self) -> Amount {
        self.amount_held
    }

    pub fn total(&self) -> Amount {
        self.amount
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}

#[cfg(test)]
mod tests {
    use crate::account::Account;

    #[test]
    fn create_account() {
        let account = Account::new(1, 42);
        assert_eq!(account.client_id, 1);
        assert_eq!(account.amount, 42);
        assert_eq!(account.amount_held, 0);
        assert_eq!(account.is_locked, false);
    }

    #[test]
    fn client_id() {
        let account = Account::new(1, 42);
        assert_eq!(account.client_id(), 1);
    }

    #[test]
    fn available() {
        let mut account = Account::new(1, 42);
        account.amount_held = 32;
        assert_eq!(account.available(), 42 - 32);
    }

    #[test]
    fn held() {
        let mut account = Account::new(1, 42);
        account.amount_held = 12;
        assert_eq!(account.held(), 12);
    }

    #[test]
    fn total() {
        let mut account = Account::new(1, 42);
        account.amount_held = 12;
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn locked() {
        let mut account = Account::new(1, 42);
        account.is_locked = true;
        assert_eq!(account.is_locked(), true);
    }

    #[test]
    fn deposit_normal() {
        let mut account = Account::new(1, 42);
        account.deposit(12);
        assert_eq!(account.available(), 42 + 12);
        assert_eq!(account.total(), 42 + 12);
    }

    #[test]
    fn deposit_locked() {
        let mut account = Account::new(1, 42);
        account.is_locked = true;
        account.deposit(12);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn withdraw_normal() {
        let mut account = Account::new(1, 42);
        account.withdraw(12);
        assert_eq!(account.available(), 42 - 12);
        assert_eq!(account.total(), 42 - 12);
    }

    #[test]
    fn withdraw_locked() {
        let mut account = Account::new(1, 42);
        account.is_locked = true;
        account.withdraw(12);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn withdraw_insufficient_total_funds() {
        let mut account = Account::new(1, 42);
        account.withdraw(80);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn withdraw_insufficient_available_funds() {
        let mut account = Account::new(1, 42);
        account.amount_held = 32;
        account.withdraw(40);
        assert_eq!(account.available(), 42 - 32);
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn dispute_normal() {
        let mut account = Account::new(1, 42);
        account.dispute(12);
        assert_eq!(account.available(), 30);
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn dispute_insufficient_available_funds() {
        let mut account = Account::new(1, 42);
        account.dispute(12);
        account.dispute(42);
        assert_eq!(account.available(), 30);
        assert_eq!(account.total(), 42);
    }
}