use std::collections::HashMap;

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::order::{OrderBase, Side};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct AccountId {
    account_id: Uuid,
}

impl AccountId {
    fn new(account: &Account) -> AccountId {
        AccountId {
            account_id: account.id,
        }
    }
    pub fn as_uuid(self) -> Uuid {
        self.account_id
    }
}

#[derive(Debug)]
pub struct Account {
    id: Uuid,
    account_balance: NotNan<f64>,
    position: i32,
}

impl Account {
    fn new(account_balance: NotNan<f64>, position: i32) -> Account {
        Account {
            id: Uuid::new_v4(),
            account_balance,
            position,
        }
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub fn view(&self) -> AccountView {
        AccountView {
            id: self.id.to_string(),
            account_balance: self.account_balance.into_inner(),
            position: self.position,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountView {
    pub id: String,
    pub account_balance: f64,
    pub position: i32,
}

#[derive(Debug, Default)]
pub struct Accounts {
    accounts: HashMap<Uuid, Account>,
}

impl Accounts {
    pub fn check_sufficient_balance(
        &self,
        account_id: AccountId,
        order: &OrderBase
    ) -> bool {
        let account = self.get(&account_id);
        match order.side {
            Side::Bid => {
                let requirement = order.limit * order.quantity as f64;
                if account.account_balance < requirement {
                    return false;
                }
            }
            Side::Ask => {
                let difference = account.position - order.quantity as i32;
                if difference < 0 {
                    let requirement = order.limit * difference as f64 * 0.5;
                    
                    if account.account_balance < requirement {
                        return false;
                    }
                } 
            }
        }
        true
    }
    pub fn create_new_account(&mut self, account_balance: NotNan<f64>, position: i32) -> AccountId {
        let account = Account::new(account_balance, position);
        let accountId = AccountId::new(&account);
        self.accounts.insert(accountId.as_uuid(), account);

        accountId
    }
    pub fn check_uuid(&self, uuid: Uuid) -> Option<AccountId> {
        let Some(account) = self.accounts.get(&uuid) else {
            return None;
        };
        Some(AccountId::new(&account))
    }
    pub fn get(&self, account_id: &AccountId) -> &Account {
        self.accounts.get(&account_id.as_uuid()).expect(
            "AccountId and Account can only be created within account module \
            or via Accounts::create_new_account which inserts the corresponding AccountId and Account \
            Account's cannot be deleted from Accounts"
                )
        // Although later on we may want to implement Account deletion. Which would screw up the above expect
        // In that case, we will need to make AccountId no longer Copy and Clone
        // And we need to implement a complicated destructor that checks there are no existing orders related to the account
        // realisation: the /account DELETE route can refuse unless the client has deleted all existing orders. So it pushes some of the responsiblity on the client
        // The destructor needs to also delete any AccountId associated. Wow this is complicated
    }
    pub fn handle_transaction(
        &mut self,
        aggressor_id: AccountId,
        counterparty_id: AccountId,
        side: Side,
        limit: f64,
        quantity: usize,
    ) {
        let aggressor = &mut self.accounts.get_mut(&aggressor_id.as_uuid()).unwrap();
        aggressor.position += (quantity as i32) * (side as i32);
        aggressor.account_balance -= (quantity as f64) * limit * (side as i32) as f64;

        let counterparty = &mut self.accounts.get_mut(&counterparty_id.as_uuid()).unwrap();
        counterparty.position -= (quantity as i32) * (side as i32);
        counterparty.account_balance += (quantity as f64) * limit * (side as i32) as f64;
    }
}

//SHOULD DO: implement Drop for Account to clean up any orders in the orderbook and refuse any orders tied to Account. Not important now, because we're not going to delete an Account.

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::NotNan;

    #[test]
    fn accounts_create_new_account() {
        let mut accounts = Accounts::default();

        let account1_id = accounts.create_new_account(NotNan::new(1e5).unwrap(), 10);
        let account2_id = accounts.create_new_account(NotNan::new(1e5).unwrap(), 0);

        assert_eq!(account1_id.as_uuid(), accounts.get(&account1_id).id);
        assert_eq!(account2_id.as_uuid(), accounts.get(&account2_id).id);
    }
}
