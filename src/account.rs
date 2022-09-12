use crate::common_types::Amount;
use log::{warn, error};

pub struct Account {
    amount: Amount,
    amount_held: Amount,
    is_locked: bool
}

impl Account {
    pub fn new(amount: Amount) -> Self {
        Self {
            amount,
            amount_held: 0,
            is_locked: false
        }
    }

    pub fn deposit(&mut self, amount: Amount) -> bool {
        if !self.is_locked {
            self.amount += amount;
            true
        } else {
            warn!("Attempt to trigger a deposit account action on a locked account");
            false
        }
    }

    pub fn withdraw(&mut self, amount: Amount) -> bool {
        if amount <= self.available() && !self.is_locked() {
            self.amount -= amount;
            true
        } else {
            warn!("Attempt to trigger a withdrawal account action on a locked account");
            false
        }
    }

    pub fn dispute(&mut self, amount: Amount) -> bool {
        if amount <= self.available() {
            self.amount_held += amount;
            true
        } else {
            error!("Attempt to trigger a dispute account action without enough funds");
            false
        }
    }

    pub fn resolve(&mut self, amount: Amount) -> bool {
        if amount <= self.held() {
            self.amount_held -= amount;
            true
        } else {
            error!("Attempt to trigger a resolve account action without enough held funds");
            false
        }
    }

    pub fn chargeback(&mut self, amount: Amount) -> bool {
        if amount <= self.held() {
            self.is_locked = true;
            self.amount_held -= amount;
            self.amount -= amount;
            true
        } else {
            error!("Attempt to trigger a chargeback account action without enough held funds");
            false
        }
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
        let account = Account::new(42);
        assert_eq!(account.amount, 42);
        assert_eq!(account.amount_held, 0);
        assert_eq!(account.is_locked, false);
    }

    #[test]
    fn available() {
        let mut account = Account::new(42);
        account.amount_held = 32;
        assert_eq!(account.available(), 42 - 32);
    }

    #[test]
    fn held() {
        let mut account = Account::new(42);
        account.amount_held = 12;
        assert_eq!(account.held(), 12);
    }

    #[test]
    fn total() {
        let mut account = Account::new(42);
        account.amount_held = 12;
        assert_eq!(account.total(), 42);
    }

    #[test]
    fn locked() {
        let mut account = Account::new(42);
        account.is_locked = true;
        assert_eq!(account.is_locked(), true);
    }

    #[test]
    fn deposit_normal() {
        let mut account = Account::new(42);
        let res = account.deposit(12);
        assert_eq!(account.available(), 42 + 12);
        assert_eq!(account.total(), 42 + 12);
        assert_eq!(res, true);
    }

    #[test]
    fn deposit_locked() {
        let mut account = Account::new(42);
        account.is_locked = true;
        let res = account.deposit(12);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }

    #[test]
    fn withdraw_normal() {
        let mut account = Account::new(42);
        let res = account.withdraw(12);
        assert_eq!(account.available(), 42 - 12);
        assert_eq!(account.total(), 42 - 12);
        assert_eq!(res, true);
    }

    #[test]
    fn withdraw_locked() {
        let mut account = Account::new(42);
        account.is_locked = true;
        let res = account.withdraw(12);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }

    #[test]
    fn withdraw_insufficient_total_funds() {
        let mut account = Account::new(42);
        let res = account.withdraw(80);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }

    #[test]
    fn withdraw_insufficient_available_funds() {
        let mut account = Account::new(42);
        account.amount_held = 32;
        let res = account.withdraw(40);
        assert_eq!(account.available(), 42 - 32);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }

    #[test]
    fn dispute_normal() {
        let mut account = Account::new(42);
        let res = account.dispute(12);
        assert_eq!(account.available(), 30);
        assert_eq!(account.total(), 42);
        assert_eq!(res, true);
    }

    #[test]
    fn dispute_insufficient_available_funds() {
        let mut account = Account::new(42);
        account.dispute(12);
        let res = account.dispute(42);
        assert_eq!(account.available(), 30);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }

    #[test]
    fn resolve_normal() {
        let mut account = Account::new(42);
        account.dispute(12);
        let res = account.resolve(8);
        assert_eq!(account.held(), 4);
        assert_eq!(res, true);
    }

    #[test]
    fn resolve_insufficient_held_funds() {
        let mut account = Account::new(42);
        account.dispute(6);
        let res = account.resolve(10);
        assert_eq!(account.held(), 6);
        assert_eq!(res, false);
    }

    #[test]
    fn chargeback_normal() {
        let mut account = Account::new(42);
        account.dispute(12);
        let res = account.chargeback(12);
        assert_eq!(account.is_locked(), true);
        assert_eq!(account.available(), 30);
        assert_eq!(account.total(), 30);
        assert_eq!(res, true);
    }

    #[test]
    fn chargeback_insufficient_held_funds() {
        let mut account = Account::new(42);
        let res = account.chargeback(12);
        assert_eq!(account.is_locked(), false);
        assert_eq!(account.available(), 42);
        assert_eq!(account.total(), 42);
        assert_eq!(res, false);
    }
}