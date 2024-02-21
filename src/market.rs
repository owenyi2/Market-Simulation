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
                    if counter.limit * f64::from(side as i32) > order.limit * f64::from(side as i32)
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

        let mut aggressor = &mut self.accounts.get_mut(&aggressor_id).expect("Account.id and Order.account_id should both private so should not go out of sync. This can still fail if we delete an Account before deleting all outstanding orders. But we haven't implemented delete yet so we're fine for now. I just realised the other way this fails is if we created an account but haven't added it to market.accounts");
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
    use ordered_float::NotNan;
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
    #[test]
    fn process_orders_1() {
        let mut market = Market::default();

        let alice = Account::new(1e5, 0);
        let bob = Account::new(1e5, 0);

        let alice_uuid = alice.get_id();
        let bob_uuid = bob.get_id();

        market.add_new_account(alice);
        market.add_new_account(bob);

        let ask1 = OrderBase::build(
            20.,
            10,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();
        let ask2 = OrderBase::build(
            30.,
            20,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();
        let ask3 = OrderBase::build(
            15.,
            1,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();
        let ask4 = OrderBase::build(
            20.,
            30,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();

        let ask2_id = ask2.get_id();
        let ask4_id = ask4.get_id();

        market.handle_incoming_order(ask1);
        market.handle_incoming_order(ask2);
        market.handle_incoming_order(ask3);
        market.handle_incoming_order(ask4);

        let bid1 =
            OrderBase::build(21., 23, Side::Bid, &market.accounts.get(&bob_uuid).unwrap()).unwrap();

        market.handle_incoming_order(bid1);

        println!("{:#?}", market.order_book);
        let best_ask = market.order_book.pop_best_counter(Side::Bid).unwrap();

        assert_eq!(best_ask.limit, 20.0.into());
        assert_eq!(best_ask.quantity, 18);
        assert_eq!(best_ask.get_id(), ask4_id);

        let best_ask = market.order_book.pop_best_counter(Side::Bid).unwrap();

        assert_eq!(best_ask.limit, 30.0.into());
        assert_eq!(best_ask.quantity, 20);
        assert_eq!(best_ask.get_id(), ask2_id);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));
    }
    #[test]
    fn process_orders_2() {
        let mut market = Market::default();

        let alice = Account::new(1e5, 0);
        let bob = Account::new(1e5, 0);
        let charlie = Account::new(1e5, 0);

        let alice_uuid = alice.get_id();
        let bob_uuid = bob.get_id();
        let charlie_uuid = charlie.get_id();

        market.add_new_account(alice);
        market.add_new_account(bob);
        market.add_new_account(charlie);

        let bid1 = OrderBase::build(
            121.5,
            20,
            Side::Bid,
            &market.accounts.get(&bob_uuid).unwrap(),
        )
        .unwrap();
        // bid1 causes no transaction
        let bid2 = OrderBase::build(
            121.5,
            20,
            Side::Bid,
            &market.accounts.get(&bob_uuid).unwrap(),
        )
        .unwrap();
        // bid2 causes no transaction
        let ask1 = OrderBase::build(
            121.9,
            10,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();
        // ask1 causes no transaction
        let ask2 = OrderBase::build(
            120.1,
            3,
            Side::Ask,
            &market.accounts.get(&alice_uuid).unwrap(),
        )
        .unwrap();
        // ask2 is cleared and consumes 3 of bid1
        let bid3 = OrderBase::build(
            122.0,
            12,
            Side::Bid,
            &market.accounts.get(&bob_uuid).unwrap(),
        )
        .unwrap();
        // bid3 consumes 10 of ask1 (clearing it)
        let ask3 = OrderBase::build(
            119.0,
            38,
            Side::Ask,
            &market.accounts.get(&charlie_uuid).unwrap(),
        )
        .unwrap();
        // ask3 first consumes 17 of bid1 (clearing it). then consumes 1 of bid3 (clearing it). then consumes 19 of bid2.

        let bid2_id = bid2.get_id();

        market.handle_incoming_order(bid1);
        market.handle_incoming_order(bid2);
        market.handle_incoming_order(ask1);
        market.handle_incoming_order(ask2);
        market.handle_incoming_order(bid3);
        market.handle_incoming_order(ask3);

        let best_bid = market.order_book.pop_best_counter(Side::Ask).unwrap();

        assert_eq!(best_bid.get_id(), bid2_id);
        assert_eq!(best_bid.limit, 121.5.into());
        assert_eq!(best_bid.quantity, 1);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));
    }
    #[test]
    fn process_orders_3() {
        let mut market = Market::default();

        let alice = Account::new(1e5, 0);
        let bob = Account::new(1e5, 0);
        let charlie = Account::new(1e5, 1000);
        let dan = Account::new(1e5, 1000);

        let alice_uuid = alice.get_id();
        let bob_uuid = bob.get_id();
        let charlie_uuid = charlie.get_id();
        let dan_uuid = dan.get_id();

        market.add_new_account(alice);
        market.add_new_account(bob);
        market.add_new_account(charlie);
        market.add_new_account(dan);

        // Alice sets up the following:
        // - 30 @ 60.01 bid / 12 @ 60.11 ask
        market.handle_incoming_order(
            OrderBase::build(
                60.01,
                30,
                Side::Bid,
                &market.accounts.get(&alice_uuid).unwrap(),
            )
            .unwrap(),
        );
        market.handle_incoming_order(
            OrderBase::build(
                60.11,
                12,
                Side::Ask,
                &market.accounts.get(&alice_uuid).unwrap(),
            )
            .unwrap(),
        );

        // Bob sets up the following:
        // - 100 @ 60.08 bid / 10 @ 60.20 ask
        market.handle_incoming_order(
            OrderBase::build(
                60.08,
                100,
                Side::Bid,
                &market.accounts.get(&bob_uuid).unwrap(),
            )
            .unwrap(),
        );
        market.handle_incoming_order(
            OrderBase::build(
                60.20,
                10,
                Side::Ask,
                &market.accounts.get(&bob_uuid).unwrap(),
            )
            .unwrap(),
        );

        // Alice sets up the following:
        // - 15 @ 60.02 bid / 14 @ 60.08 ask
        market.handle_incoming_order(
            OrderBase::build(
                60.02,
                15,
                Side::Bid,
                &market.accounts.get(&alice_uuid).unwrap(),
            )
            .unwrap(),
        );
        market.handle_incoming_order(
            OrderBase::build(
                60.08,
                14,
                Side::Ask,
                &market.accounts.get(&alice_uuid).unwrap(),
            )
            .unwrap(),
        );

        // Charlie sets up the following:
        // - 120 @ 60.01 ask
        market.handle_incoming_order(
            OrderBase::build(
                60.01,
                120,
                Side::Ask,
                &market.accounts.get(&charlie_uuid).unwrap(),
            )
            .unwrap(),
        );

        // Dan sets up the following
        // - 20 @ 60.10 bid / 10 @ 60.3 ask
        market.handle_incoming_order(
            OrderBase::build(
                60.11,
                20,
                Side::Bid,
                &market.accounts.get(&dan_uuid).unwrap(),
            )
            .unwrap(),
        );
        market.handle_incoming_order(
            OrderBase::build(
                60.3,
                10,
                Side::Ask,
                &market.accounts.get(&dan_uuid).unwrap(),
            )
            .unwrap(),
        );

        let alice_account = market.accounts.get(&alice_uuid).unwrap();
        let bob_account = market.accounts.get(&bob_uuid).unwrap();
        let charlie_account = market.accounts.get(&charlie_uuid).unwrap();
        let dan_account = market.accounts.get(&dan_uuid).unwrap();

        println!("{:#?}", &alice_account);
        println!("{:#?}", &bob_account);
        println!("{:#?}", &charlie_account);
        println!("{:#?}", &dan_account);
        println!("{:#?}", market.order_book);

        let best_ask = market.order_book.pop_best_counter(Side::Bid).unwrap();

        assert_eq!(best_ask.limit, 60.2.into());
        assert_eq!(best_ask.quantity, 10);

        let best_ask = market.order_book.pop_best_counter(Side::Bid).unwrap();

        assert_eq!(best_ask.limit, 60.3.into());
        assert_eq!(best_ask.quantity, 10);

        let best_bid = market.order_book.pop_best_counter(Side::Ask).unwrap();

        assert_eq!(best_bid.limit, 60.11.into());
        assert_eq!(best_bid.quantity, 8);

        let best_bid = market.order_book.pop_best_counter(Side::Ask).unwrap();

        assert_eq!(best_bid.limit, 60.01.into());
        assert_eq!(best_bid.quantity, 11);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));

        assert_eq!(alice_account.account_balance, 99522.1.into());
        assert_eq!(alice_account.position, 8);
        assert_eq!(bob_account.account_balance, 93998.02.into());
        assert_eq!(bob_account.position, 100);
        assert_eq!(charlie_account.account_balance, 107201.2.into());
        assert_eq!(charlie_account.position, 880);
        assert_eq!(dan_account.account_balance, 99278.68.into());
        assert_eq!(dan_account.position, 1012);

    }
}
