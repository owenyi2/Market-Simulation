use std::cmp::{min, Ordering};
use std::ops::Neg;

use ordered_float::NotNan;
use uuid::Uuid;

use crate::account::AccountId;

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

#[derive(Debug)]
pub struct OrderBase {
    limit: NotNan<f64>,
    timestamp: NotNan<f64>,
    pub quantity: usize,
    pub side: Side,
    account_id: AccountId,
    id: Uuid,
}

impl PartialEq for OrderBase {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for OrderBase {}

#[derive(Debug)]
pub struct AskOrder {
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
pub struct BidOrder {
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

pub trait IntoOrderBase {
    fn into_order_base(self) -> OrderBase; 
}

impl IntoOrderBase for BidOrder {
    fn into_order_base(self) -> OrderBase {
        self.order
    }
}

impl IntoOrderBase for AskOrder {
    fn into_order_base(self) -> OrderBase {
        self.order
    }
}
// so that Market.order_book is a HashMap<Side, BinaryHeap<Order>>

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account;
    use uuid::Uuid;

    #[test]
    fn AskOrdering() {
        let account = account::Account::new(1e5, 0);

        let ask1 = AskOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 12,
                side: Side::Ask,
                account_id: AccountId::new(&account),
                id: Uuid::new_v4(),
            },
        };
        let ask2 = AskOrder {
            order: OrderBase {
                limit: NotNan::new(0.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 10,
                side: Side::Ask,
                account_id: AccountId::new(&account),
                id: Uuid::new_v4(),
            },
        };
        // ask2 should be higher priority than ask1 because it has a lower limit price
        assert!(ask2 > ask1);
    }
    #[test]
    fn BidOrdering() {
        let account = account::Account::new(1e5, 0);

        let bid1 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 2,
                side: Side::Bid,
                account_id: AccountId::new(&account),
                id: Uuid::new_v4(),
            },
        };
        let bid2 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(4.).unwrap(),
                quantity: 3,
                side: Side::Bid,
                account_id: AccountId::new(&account),
                id: Uuid::new_v4(),
            },
        };
        let bid3 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(3.).unwrap(),
                timestamp: NotNan::new(3.).unwrap(),
                quantity: 2,
                side: Side::Bid,
                account_id: AccountId::new(&account),
                id: Uuid::new_v4(),
            },
        };
        let bid4 = BidOrder {
            order: OrderBase {
                limit: NotNan::new(2.).unwrap(),
                timestamp: NotNan::new(4.).unwrap(),
                quantity: 3,
                side: Side::Bid,
                account_id: AccountId::new(&account),
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
}
