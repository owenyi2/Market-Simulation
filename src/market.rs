use std::cmp::{min, Ordering};
use std::collections::{BinaryHeap, HashMap};

use uuid::Uuid;

use crate::account::{Account, AccountId};
use crate::order::{self, OrderBase, OrderBook, Side};

#[derive(Debug, Default)]
struct Market {
    order_book: OrderBook,
    accounts: HashMap<Uuid, Account>,
}

impl Market {
    fn add_new_account(&mut self, account: Account) {
        self.accounts.insert(account.get_id(), account);
    }

    fn handle_incoming_order(&mut self, mut order: OrderBase) {
        let side = order.side;

        let order = loop {
            let best_counter = self.order_book.get_best_counter(side);
            match best_counter {
                Some(counter) => {
                    if counter.limit * f64::from(side as i32) < order.limit * f64::from(side as i32)
                    {
                        break Some(order);
                    }
                }
                None => break Some(order),
            }

            let mut matched = self.order_book.pop_best_counter(side).unwrap();
            let aggressor_id = order.account_id;
            let counterparty_id = matched.account_id;
            let transaction_quantity = min(order.quantity, matched.quantity);

            if matched.quantity == transaction_quantity {
            } else {
                self.handle_transaction(
                    aggressor_id,
                    counterparty_id,
                    side,
                    f64::from(order.limit),
                    transaction_quantity,
                );

                matched.quantity -= transaction_quantity;
                self.order_book.insert_order(matched);
            }
            if order.quantity == transaction_quantity {
                break None;
            } else {
                self.handle_transaction(
                    aggressor_id,
                    counterparty_id,
                    side,
                    f64::from(order.limit),
                    transaction_quantity,
                );
                order.quantity -= transaction_quantity;
            }
        };
        if let Some(order) = order {
            self.order_book.insert_order(order);
        }
    }
    fn handle_transaction(
        &mut self,
        aggressor_id: AccountId,
        counterparty_id: AccountId,
        side: Side,
        limit: f64,
        quantity: usize,
    ) {
        let aggressor_id = aggressor_id.as_uuid();
        let counterparty_id = counterparty_id.as_uuid();

        let mut aggressor = &mut self.accounts.get_mut(&aggressor_id).expect("Account.id and Order.account_id should both private so should not go out of sync. This can still fail if we delete an Account before deleting all outstanding orders. But we haven't implemented delete yet so we're fine for now");
        aggressor.position += (quantity as i32) * (side as i32);
        aggressor.account_balance -= (quantity as f64) * limit * (side as i32) as f64;

        let mut counterparty = &mut self
            .accounts
            .get_mut(&counterparty_id)
            .expect("See above expect message");

        counterparty.position -= (quantity as i32) * (side as i32);
        counterparty.account_balance += (quantity as f64) * limit * (side as i32) as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn Market_add_new_account() {
        let mut market = Market::default();

        let account1 = Account::new(1e5, 0);
        let account2 = Account::new(1e4, 0);

        let account1_id = account1.get_id();
        let account2_id = account2.get_id();

        market.add_new_account(account1);
        market.add_new_account(account2);

        // check that the hashmap has stored it
        let a1 = market.accounts.get(&account1_id).unwrap();
        let a2 = market.accounts.get(&account2_id).unwrap();

        assert_eq!(a1.get_id(), account1_id);
        assert_eq!(a2.get_id(), account2_id);

        //and the id in the key and id in the value match
        //which i mean of course it does. what's the point of this test
    }
}
