// use std::cmp::{min, Ordering};
// use std::collections::{BinaryHeap, HashMap};
// 
// use uuid::Uuid;
// 
// use crate::order::{self, Side, OrderBase, BidOrder, AskOrder};
// use crate::account::{AccountId, Account};
// 
// // Solution for our woes
// //  - remove the AskOrder and BidOrder
// //  - Create a struct around a BinaryHeap called OrderHeap or smth
// //  - The orderheap has a field called side and has insert, get, peek etc. that wrap around the BinaryHeap methods, but guards against placing asks into a heap of bids
// //  - This doesn't work because the ordering for Asks and Bids is not the same but also not anti (can't just shove a reverse).
// //  - the hacky workaround is to make the implementation for Ord secretly a Partial Ord that panics if it tries to compare a bid with an ask and then guarantee in future code, that bids and asks are never compared
// // Another possible solution
// // Implement the IntoOrderBase trait,
// // Use a. Jokes doesn't work because that's still 2 different types. Double Jokes, but it is fine
// 
// // maybe we should move OrderBook to order.rs, have AskOrder and BidOrder be internal shit. Have interfaces that convert them down to just Order. This could be combined with 1st idea to make a fragile Ord because it allows us to encapsulate the fragility into order.rs module
// 
// #[derive(Debug, Default)]
// struct OrderBook {
//     bid_heap: BinaryHeap<BidOrder>,
//     ask_heap: BinaryHeap<AskOrder>,
// }
// impl OrderBook {
//     fn get<T: order::IntoOrderBase> (&self, side: Side) -> BinaryHeap<T> {
//         if side == Side::Bid {
//             return self.bid_heap;
//         }
//         if side == Side::Ask {
//             return self.ask_heap;
//         }
//     }
// }
// 
// #[derive(Debug, Default)]
// struct Market {
//     order_book: OrderBook, 
//     accounts: HashMap<Uuid, Account>
// }
// 
// impl Market {
//     fn add_new_account(&mut self, account: Account) {
//         self.accounts.insert(account.get_id(), account);
//     }
//     fn handle_incoming_order(&mut self, mut order: OrderBase) {
//         let side = order.side;
//         let other_side = -side; 
// 
//         let order = loop {
//         };
// 
//         //println!("{:?}", order.order);
//     }
// }
// 
// 
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use uuid::Uuid;
// 
//     #[test]
//     fn Market_add_new_account() {
//         let mut market = Market::default(); 
// 
// 
//         
//         
//         let account1 = Account::new(1e5, 0);
//         let account2 = Account::new(1e4, 0);
// 
//         let account1_id = account1.get_id();
//         let account2_id = account2.get_id();
// 
//         market.add_new_account(account1);
//         market.add_new_account(account2);
// 
//         // check that the hashmap has stored it
//         let a1 = market.accounts.get(&account1_id).unwrap();
//         let a2 = market.accounts.get(&account2_id).unwrap();
// 
//         assert_eq!(a1.get_id(), account1_id);
//         assert_eq!(a2.get_id(), account2_id);
// 
//         //and the id in the key and id in the value match
//         //which i mean of course it does. what's the point of this test
//     }
// }
