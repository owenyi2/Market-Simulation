use std::cmp::{Ordering};
use std::error::Error;
use std::ops::Neg;
use std::time::{SystemTime, UNIX_EPOCH};

use ordered_float::NotNan;
use uuid::Uuid;
use keyed_priority_queue::KeyedPriorityQueue;

use crate::account::{Account, AccountId};

// consider pub (super)

#[derive(Debug, Default)]
pub struct OrderBook {
    bids: KeyedPriorityQueue<OrderId, BidOrder>,
    asks: KeyedPriorityQueue<OrderId, AskOrder>,
}

impl OrderBook {
    pub fn peek(&self, side: Side) -> Option<&OrderBase> {
        match side {
            Side::Ask => Some(&self.asks.peek()?.1.order),
            Side::Bid => Some(&self.bids.peek()?.1.order),
        }
    }
    // this is a bit counterintuitive of an API, we should do pop_best and get_best with side
    // if the called wants the opposite they should use -side because we impl Neg already
    pub fn pop(&mut self, side: Side) -> Option<OrderBase> {
        match side {
            Side::Ask => Some(self.asks.pop()?.1.order),
            Side::Bid => Some(self.bids.pop()?.1.order),
        }
    }
    pub fn insert_order(&mut self, order: OrderBase) {
        match order.side {
            Side::Ask => { self.asks.push(OrderId::new(&order), AskOrder { order }); },
            Side::Bid => { self.bids.push(OrderId::new(&order), BidOrder { order }); },
        }
    }
    pub fn is_empty(&self, side: Side) -> bool {
        match side {
            Side::Ask => self.asks.is_empty(),
            Side::Bid => self.bids.is_empty(),
        }
    }
    pub fn delete_order(&mut self, order_id: OrderId) -> Option<OrderBase> {
        if let Some(ask_order) = self.asks.remove(&order_id) {
            return Some(ask_order.order);
        }
        if let Some(bid_order) = self.bids.remove(&order_id) {
            return Some(bid_order.order);
        }
        None  
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Side {
    Ask = -1,
    Bid = 1,
}

impl Neg for Side {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Side::Ask => Side::Bid,
            Side::Bid => Side::Ask,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct OrderId {
    order_id: Uuid
}
impl OrderId {
    fn new(order: &OrderBase) -> OrderId {
        OrderId { order_id: order.id }
    }
}

#[derive(Debug)]
pub struct OrderBase {
    pub limit: NotNan<f64>,
    timestamp: NotNan<f64>,
    pub quantity: usize,
    pub side: Side,
    pub account_id: AccountId,
    id: Uuid,
}

// Make this a builder instead of a new
impl OrderBase {
    pub fn build(
        limit: f64,
        quantity: usize,
        side: Side,
        account_id: AccountId,
    ) -> Result<OrderBase, Box<dyn Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        Ok(OrderBase {
            limit: NotNan::new(limit)?,
            timestamp: NotNan::new(timestamp)?,
            quantity,
            side,
            account_id,
            id: Uuid::new_v4(),
        })
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

impl PartialEq for OrderBase {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for OrderBase {}

#[derive(Debug)]
struct AskOrder {
    order: OrderBase,
}

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.order.limit == other.order.limit {
            return other.order.timestamp.cmp(&self.order.timestamp);
        }
        other.order.limit.cmp(&self.order.limit)
    }
}

impl PartialOrd for AskOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AskOrder {
    fn eq(&self, other: &Self) -> bool {
        (self.order.limit == other.order.limit) && (self.order.timestamp == other.order.timestamp)
    }
}

impl Eq for AskOrder {}

#[derive(Debug)]
struct BidOrder {
    order: OrderBase,
}

impl Ord for BidOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.order.limit == other.order.limit {
            return other.order.timestamp.cmp(&self.order.timestamp);
        }
        self.order.limit.cmp(&other.order.limit)
    }
}

impl PartialOrd for BidOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BidOrder {
    fn eq(&self, other: &Self) -> bool {
        (self.order.limit == other.order.limit) && (self.order.timestamp == other.order.timestamp)
    }
}

impl Eq for BidOrder {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account;
    use uuid::Uuid;
    use crate::market::Market;

    #[test]
    fn ask_ordering() {
        let mut market = Market::default();

        let account_id = market.accounts.create_new_account(NotNan::new(1e5).unwrap(), 0);

        let ask1 = AskOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 12,
                side: Side::Ask,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        let ask2 = AskOrder {
            order: OrderBase {
                limit: NotNan::new(0.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 10,
                side: Side::Ask,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        // ask2 should be higher priority than ask1 because it has a lower limit price
        assert!(ask2 > ask1);
    }
    #[test]
    fn bid_ordering() { 
        let mut market = Market::default();

        let account_id = market.accounts.create_new_account(NotNan::new(1e5).unwrap(), 0);

        let bid1 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 2,
                side: Side::Bid,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        let bid2 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(4.).unwrap(),
                quantity: 3,
                side: Side::Bid,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        let bid3 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(3.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 2,
                side: Side::Bid,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        let bid4 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(4.).unwrap(),
                quantity: 3,
                side: Side::Bid,
                account_id: account_id,
                id: Uuid::new_v4(),
            },
        };
        // bid1 has same limit price as bid2 but bid1 was submitted earlier
        assert!(bid1 > bid2);
        // bid2 and bid4 have the same price-time priority
        assert!(bid2 == bid4);
        // bid3 offers a higher price than bid4
        assert!(bid3 > bid4);
    }
    #[test]
    fn order_base_builder() { 
        let mut market = Market::default();

        let account_id = market.accounts.create_new_account(NotNan::new(1e5).unwrap(), 0);

        let ask1 = OrderBase::build(20., 10, Side::Ask, account_id).unwrap();
        let ask2 = OrderBase::build(30., 20, Side::Ask, account_id).unwrap();
        let ask3 = OrderBase::build(15., 1, Side::Ask, account_id).unwrap();
        let ask4 = OrderBase::build(20., 30, Side::Ask, account_id).unwrap();

        println!("Ask_1: {:?}", ask1);
        println!("Ask_2: {:?}", ask2);
        println!("Ask_3: {:?}", ask3);
        println!("Ask_4: {:?}", ask4);

        assert!(ask1.timestamp <= ask2.timestamp);
        assert!(ask2.timestamp <= ask3.timestamp);
        assert!(ask3.timestamp <= ask4.timestamp);
    }
    #[test]
    fn is_empty() {
        let order_book = OrderBook::default();
        assert!(order_book.is_empty(Side::Ask));
        assert!(order_book.is_empty(Side::Bid));
    }
    #[test]
    fn order_book_priority() {
        let mut market = Market::default();

        let account_id = market.accounts.create_new_account(NotNan::new(1e5).unwrap(), 0);

        let mut order_book = OrderBook::default();

        let ask1 = OrderBase {
            limit: NotNan::new(20.).unwrap(),
            timestamp: NotNan::new(1703713624.0).unwrap(),
            quantity: 10,
            side: Side::Ask,
            account_id: account_id, 
            id: Uuid::new_v4(),
        };
        let ask2 = OrderBase {
            limit: NotNan::new(30.).unwrap(),
            timestamp: NotNan::new(1703713626.0).unwrap(),
            quantity: 20,
            side: Side::Ask,
            account_id: account_id,
            id: Uuid::new_v4(),
        };
        let ask3 = OrderBase {
            limit: NotNan::new(15.).unwrap(),
            timestamp: NotNan::new(1703713628.0).unwrap(),
            quantity: 1,
            side: Side::Ask,
            account_id: account_id,
            id: Uuid::new_v4(),
        };
        let ask4 = OrderBase {
            limit: NotNan::new(20.).unwrap(),
            timestamp: NotNan::new(1703713629.0).unwrap(),
            quantity: 30,
            side: Side::Ask,
            account_id: account_id,
            id: Uuid::new_v4(),
        };

        let (ask1_id, ask2_id, ask3_id, ask4_id) = (ask1.id, ask2.id, ask3.id, ask4.id);

        order_book.insert_order(ask1);
        order_book.insert_order(ask2);
        order_book.insert_order(ask3);
        order_book.insert_order(ask4);

        assert_eq!(order_book.peek(Side::Ask).unwrap().id, ask3_id);

        assert_eq!(order_book.pop(Side::Ask).unwrap().id, ask3_id);
        assert_eq!(order_book.pop(Side::Ask).unwrap().id, ask1_id);
        assert_eq!(order_book.pop(Side::Ask).unwrap().id, ask4_id);
        assert_eq!(order_book.pop(Side::Ask).unwrap().id, ask2_id);
    }
}
