use std::collections::HashMap;

use ordered_float::NotNan;
use uuid::Uuid;

use super::order::Side; 


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
    // It makes more sense to only create an AccountId if the account has been stored in a market. So the constructor should take a uuid, validate if the uuid is in the market.accounts HashMap then return a Result rather than an AccountId
    // TODO: Refactor as per above. Jokes, our unit tests are not really unit tests e.g. unit tests for Order need an account instance to create an Order. To solve this we could make orderbase an actual order base, then derive other stuff on top but that's fucked + a lot of code edits are propagated from that. ok this refactor if done, will require a market instance in the order unit test. Oh well.
    fn as_uuid(self) -> Uuid {
        self.account_id
    }
}

#[derive(Debug)]
pub struct Account {
    id: Uuid,
    pub account_balance: NotNan<f64>,
    pub position: i32,
}

impl Account {
    pub fn new(account_balance: NotNan<f64>, position: i32) -> Account {
        Account {
            id: Uuid::new_v4(),
            account_balance,
            position,
        }
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Default)]
pub struct Accounts {
    accounts: HashMap<AccountId, Account>
}

impl Accounts {
    pub fn create_new_account(&mut self, account_balance: NotNan<f64>, position: i32) -> AccountId {
        let account = Account::new(account_balance, position);
        let accountId = AccountId::new(&account);
        self.accounts.insert(accountId, account);

        accountId
    }
    pub fn get(&self, account_id: &AccountId) -> &Account {
        self.accounts.get(account_id).expect(
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
    pub fn handle_transaction(&mut self, aggressor_id: AccountId, counterparty_id: AccountId, side: Side, limit: f64, quantity: usize) {
        let aggressor = &mut self.accounts.get_mut(&aggressor_id).unwrap();
        aggressor.position += (quantity as i32) * (side as i32);
        aggressor.account_balance -= (quantity as f64) * limit * (side as i32) as f64;

        let counterparty = &mut self.accounts.get_mut(&counterparty_id).unwrap();
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
