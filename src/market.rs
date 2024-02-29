use std::cmp::min;

use ordered_float::NotNan;
use uuid::Uuid;

use crate::account::{Account, AccountId, Accounts};
use crate::order::{OrderBase, OrderBook, ProcessedOrders, Status};

#[derive(Debug, Default)]
pub struct Market {
    order_book: OrderBook,
    accounts: Accounts,
    processed_orders: ProcessedOrders
}

impl Market {
    pub fn new_account(
        &mut self,
        account_balance: f64,
        position: i32,
    ) -> Result<AccountId, &'static str> {
        let Ok(account_balance) = NotNan::new(account_balance) else {
            return Err("Invalid Account balance");
        };
        Ok(self.accounts.create_new_account(account_balance, position))
    }
    pub fn check_uuid(&self, uuid: Uuid) -> Option<AccountId> {
        self.accounts.check_uuid(uuid)
    }
    pub fn get_account(&self, account_id: &AccountId) -> &Account {
        self.accounts.get(&account_id)
    }
    pub fn handle_incoming_order(&mut self, mut order: OrderBase) {
        let side = order.side;
        order.status = Status::Pending;
        let order = loop {
            let best_counter = self.order_book.peek(-side);
            match best_counter {
                Some(counter) => {
                    if counter.limit * f64::from(side as i32) > order.limit * f64::from(side as i32)
                    {
                        break Some(order);
                    }
                }
                None => break Some(order),
            }

            let mut matched = self.order_book.pop(-side).unwrap();
            let aggressor_id = order.account_id;
            let counterparty_id = matched.account_id;
            let transaction_quantity = min(order.quantity, matched.quantity);
            
            self.accounts.handle_transaction(
                aggressor_id,
                counterparty_id,
                side,
                f64::from(order.limit),
                transaction_quantity,
            );

            if matched.quantity == transaction_quantity {
                matched.status = Status::Executed;
                self.processed_orders.push(matched);
            } else {
                // possible refactor:
                // have a self.handle_transaction which calls accounts.handle_transaction as one of the subtasks. I anticipate that more and more functionality will need to be implemented to properly facilitate a transaction e.g. updating the existing orders tracked by accounts. We may also want to make this async in which case, we would want to delegate processing into an await block. This function should only determine if a transaction can be made
                matched.quantity -= transaction_quantity;
                self.order_book.insert_order(matched);
            }
            if order.quantity == transaction_quantity {
                order.status = Status::Executed;
                self.processed_orders.push(order);
                break None;
            } else {
                order.quantity -= transaction_quantity;
            }
        };
        if let Some(order) = order {
            self.order_book.insert_order(order);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;

    use ordered_float::NotNan;
    #[test]
    fn process_orders_1() {
        let mut market = Market::default();

        let alice_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);
        let bob_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);

        let ask1 = OrderBase::build(20., 10, Side::Ask, alice_id).unwrap();
        let ask2 = OrderBase::build(30., 20, Side::Ask, alice_id).unwrap();
        let ask3 = OrderBase::build(15., 1, Side::Ask, alice_id).unwrap();
        let ask4 = OrderBase::build(20., 30, Side::Ask, alice_id).unwrap();

        let ask2_id = ask2.get_id();
        let ask4_id = ask4.get_id();

        market.handle_incoming_order(ask1);
        market.handle_incoming_order(ask2);
        market.handle_incoming_order(ask3);
        market.handle_incoming_order(ask4);

        let bid1 = OrderBase::build(21., 23, Side::Bid, bob_id).unwrap();

        market.handle_incoming_order(bid1);

        println!("{:#?}", market.order_book);
        let best_ask = market.order_book.pop(Side::Ask).unwrap();

        assert_eq!(best_ask.limit.into_inner(), 20.0);
        assert_eq!(best_ask.quantity, 18);
        assert_eq!(best_ask.get_id(), ask4_id);

        let best_ask = market.order_book.pop(Side::Ask).unwrap();

        assert_eq!(best_ask.limit.into_inner(), 30.0);
        assert_eq!(best_ask.quantity, 20);
        assert_eq!(best_ask.get_id(), ask2_id);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));
    }
    #[test]
    fn process_orders_2() {
        let mut market = Market::default();

        let alice_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);
        let bob_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);
        let charlie_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);

        let bid1 = OrderBase::build(121.5, 20, Side::Bid, bob_id).unwrap();
        // bid1 causes no transaction
        let bid2 = OrderBase::build(121.5, 20, Side::Bid, bob_id).unwrap();
        // bid2 causes no transaction
        let ask1 = OrderBase::build(121.9, 10, Side::Ask, alice_id).unwrap();
        // ask1 causes no transaction
        let ask2 = OrderBase::build(120.1, 3, Side::Ask, alice_id).unwrap();
        // ask2 is cleared and consumes 3 of bid1
        let bid3 = OrderBase::build(122.0, 12, Side::Bid, bob_id).unwrap();
        // bid3 consumes 10 of ask1 (clearing it)
        let ask3 = OrderBase::build(119.0, 38, Side::Ask, charlie_id).unwrap();
        // ask3 first consumes 17 of bid1 (clearing it). then consumes 1 of bid3 (clearing it). then consumes 19 of bid2.

        let bid2_id = bid2.get_id();

        market.handle_incoming_order(bid1);
        market.handle_incoming_order(bid2);
        market.handle_incoming_order(ask1);
        market.handle_incoming_order(ask2);
        market.handle_incoming_order(bid3);
        market.handle_incoming_order(ask3);

        let best_bid = market.order_book.pop(Side::Bid).unwrap();

        assert_eq!(best_bid.get_id(), bid2_id);
        assert_eq!(best_bid.limit.into_inner(), 121.5);
        assert_eq!(best_bid.quantity, 1);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));
    }
    #[test]
    fn process_orders_3() {
        let mut market = Market::default();

        let alice_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);
        let bob_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 0);
        let charlie_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 1000);
        let dan_id = market
            .accounts
            .create_new_account(NotNan::new(1e5).unwrap(), 1000);

        // Alice sets up the following:
        // - 30 @ 60.01 bid / 12 @ 60.11 ask
        market.handle_incoming_order(OrderBase::build(60.01, 30, Side::Bid, alice_id).unwrap());
        market.handle_incoming_order(OrderBase::build(60.11, 12, Side::Ask, alice_id).unwrap());

        // Bob sets up the following:
        // - 100 @ 60.08 bid / 10 @ 60.20 ask
        market.handle_incoming_order(OrderBase::build(60.08, 100, Side::Bid, bob_id).unwrap());
        market.handle_incoming_order(OrderBase::build(60.20, 10, Side::Ask, bob_id).unwrap());

        // Alice sets up the following:
        // - 15 @ 60.02 bid / 14 @ 60.08 ask
        market.handle_incoming_order(OrderBase::build(60.02, 15, Side::Bid, alice_id).unwrap());
        market.handle_incoming_order(OrderBase::build(60.08, 14, Side::Ask, alice_id).unwrap());

        // Charlie sets up the following:
        // - 120 @ 60.01 ask
        market.handle_incoming_order(OrderBase::build(60.01, 120, Side::Ask, charlie_id).unwrap());

        // Dan sets up the following
        // - 20 @ 60.10 bid / 10 @ 60.3 ask
        market.handle_incoming_order(OrderBase::build(60.11, 20, Side::Bid, dan_id).unwrap());
        market.handle_incoming_order(OrderBase::build(60.3, 10, Side::Ask, dan_id).unwrap());

        // Alice sets up the following
        // - 8 @ 60.09 ask
        market.handle_incoming_order(OrderBase::build(60.08, 8, Side::Ask, alice_id).unwrap());

        let alice_account = market.accounts.get(&alice_id);
        let bob_account = market.accounts.get(&bob_id);
        let charlie_account = market.accounts.get(&charlie_id);
        let dan_account = market.accounts.get(&dan_id);

        println!("{:#?}", &alice_account);
        println!("{:#?}", &bob_account);
        println!("{:#?}", &charlie_account);
        println!("{:#?}", &dan_account);
        println!("{:#?}", market.order_book);

        let best_ask = market.order_book.pop(Side::Ask).unwrap();

        assert_eq!(best_ask.limit.into_inner(), 60.2);
        assert_eq!(best_ask.quantity, 10);

        let best_ask = market.order_book.pop(Side::Ask).unwrap();

        assert_eq!(best_ask.limit.into_inner(), 60.3);
        assert_eq!(best_ask.quantity, 10);

        let best_bid = market.order_book.pop(Side::Bid).unwrap();

        assert_eq!(best_bid.limit.into_inner(), 60.01);
        assert_eq!(best_bid.quantity, 11);

        assert!(market.order_book.is_empty(Side::Ask));
        assert!(market.order_book.is_empty(Side::Bid));

        assert_eq!(alice_account.view().account_balance, 100002.74);
        assert_eq!(alice_account.view().position, 0);
        assert_eq!(bob_account.view().account_balance, 93998.02);
        assert_eq!(bob_account.view().position, 100);
        assert_eq!(charlie_account.view().account_balance, 107201.2);
        assert_eq!(charlie_account.view().position, 880);
        assert_eq!(dan_account.view().account_balance, 98798.04);
        assert_eq!(dan_account.view().position, 1020);
    }
}
