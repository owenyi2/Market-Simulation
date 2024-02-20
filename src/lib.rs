use ordered_float::NotNan;
use std::cmp::{min, Ordering};
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

pub mod account;
pub mod order;
pub mod market;

// #[derive(debug)]
// struct Market {
//     order_book: (BinaryHeap<AskOrder>, BinaryHeap<BidOrder>),
//     accounts: HashMap<AccountId, Account>
// }
//
// impl Market {
//     fn add_new_account(&mut self, account: Account) {
//         self.accounts.insert(account.id.clone(), account);
//     }
//
//     fn handle_incoming_order(&mut self, mut order: OrderBase) {
//         let side = order.side.clone();
//         let other_side = match side { Side::Bid => Side::Ask, Side::Ask => Side::Bid};
//
//         let order = loop { // 0. Successively match incoming `order` with `best_offer`'s working throu
// gh the priority queue
//             match other_side { // 1. Check if a transaction can be made with the incoming `order`
//                 Side::Ask => {
//                     let best_offer = self.order_book.0.peek();
//                     if best_offer == None || best_offer.unwrap().order.limit > order.limit {
//                         break Some(order); // No possible transaction to make so early exit loop
//                     }
//                 },
//                 Side::Bid => {
//                     let best_offer = self.order_book.1.peek();
//                     if best_offer == None || best_offer.unwrap().order.limit < order.limit {
//                         break Some(order); // No possible transaction to make so early exit loop
//                     }
//                 }
//             }
//
//             // 2. Otherwise continue to match incoming `order` with `best_offer`
//             let mut best_offer = match other_side {
//                 Side::Ask => self.order_book.0.pop().unwrap().order,
//                 Side::Bid => self.order_book.1.pop().unwrap().order,
//             }; // pop now to take ownership. If `best_offer` not fully consumed then modify and push i
// t back later
//
//             let aggressor_id = order.account_id.clone();
//             let best_offer_id = best_offer.account_id.clone();
//
//             let transaction_quantity = min(order.quantity, best_offer.quantity);
//             if best_offer.quantity == transaction_quantity {
//                 // pop occured before so we simply do nothing in this arm
//             } else {
//                 self.handle_transaction(&aggressor_id, &best_offer_id, &order.side, &order.limit, tran
// saction_quantity);
//
//                 best_offer.quantity -= transaction_quantity; // push back modified quantity
//                 match other_side {
//                     Side::Ask => self.order_book.0.push(AskOrder { order: best_offer }),
//                     Side::Bid => self.order_book.1.push(BidOrder { order: best_offer }),
//                 }
//             }
//
//             if order.quantity == transaction_quantity {
//                 break None;
//             } else {
//                 self.handle_transaction(&aggressor_id, &best_offer_id, &order.side, &order.limit, tran
// saction_quantity);
//
//                 order.quantity -= transaction_quantity;
//                 // then loops back to the next best_offer
//             }
//         };
//         match order { //
//             Some(order) => match side {
//                 Side::Ask => self.order_book.0.push(AskOrder { order }),
//                 Side::Bid => self.order_book.1.push(BidOrder { order }),
//             }
//             None => (),
//         }
//     }
//
//     fn handle_transaction(&mut self, aggressor_id: &AccountId, counterparty_id: &AccountId, side: &Sid
// e, limit: &NotNan<f64>, quantity: usize) {
//         let sign = match side { Side::Ask => -1, Side::Bid => 1 };
//
//         let mut aggressor = &mut self.accounts.get_mut(aggressor_id).expect("Ah shit. The `BaseOrder.a
// ccount_id` is out of sync with the `Market.accounts`.");
//         aggressor.position += <usize as AsPrimitive<i32>>::as_(quantity) * sign;
//         aggressor.account_balance -= quantity as f64 * limit.into_inner() * sign as f64;
//
//         let mut counterparty = &mut self.accounts.get_mut(counterparty_id).expect("Ah shit. The `BaseO
// rder.account_id` is out of sync with the `Market.accounts`.");
//         counterparty.position -= <usize as AsPrimitive<i32>>::as_(quantity) * sign;
//         counterparty.account_balance += quantity as f64 * limit.into_inner() * sign as f64;
//     }
// }
