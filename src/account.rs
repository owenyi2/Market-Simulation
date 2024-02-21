use ordered_float::NotNan;
use uuid::Uuid;

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct AccountId {
    account_id: Uuid,
}

impl AccountId {
    pub fn new(account: &Account) -> AccountId {
        AccountId {
            account_id: account.id,
        }
    }
    // It makes more sense to only create an AccountId if the account has been stored in a market. So the constructor should take a uuid, validate if the uuid is in the market.accounts HashMap then return a Result rather than an AccountId
    // TODO: Refactor as per above. Jokes, our unit tests are not really unit tests e.g. unit tests for Order need an account instance to create an Order. To solve this we could make orderbase an actual order base, then derive other stuff on top but that's fucked + a lot of code edits are propagated from that. ok this refactor if done, will require a market instance in the order unit test. Oh well.
    pub fn as_uuid(self) -> Uuid {
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
    pub fn new(account_balance: f64, position: i32) -> Account {
        Account {
            id: Uuid::new_v4(),
            account_balance: NotNan::new(account_balance).unwrap(),
            position,
        }
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}
//SHOULD DO: implement Drop for Account to clean up any orders in the orderbook and refuse any orders tied to Account. Not important now, because we're not going to delete an Account.
