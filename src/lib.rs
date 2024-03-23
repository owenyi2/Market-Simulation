use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{self, SystemTime, UNIX_EPOCH, Duration};
use std::cmp::{Ordering, min, max};
use std::ops::Neg;
use std::fmt::{self, Display};

use crossbeam;
use rand::Rng;
use rand_distr::{Exp, LogNormal, Bernoulli, Normal, Distribution};
use uuid::Uuid;
use ordered_float::NotNan;

// Current Problem

// We want to share memory for agents to read
// Previously, this was achieved with channels and copying account info
// Unfortunately the Orderbook does not implement Copy (because it's data lives on the heap)
// Also I misinterpreted crossbeam channel. I thought it meant that the consumers could get the same data but, actually it means multiple consumers can take successive data from the channel

// Idea

// Create another crossbeam thread inside Market for serving read requests from agents 
// Each loop in Market::run, we Clone the orderbook and send it via an mpsc to the serving thread
// - Even though cloning is ehh, I think it will still be faster than waiting on blocking Mutex's
// - though if we were bothered, we should profile both approaches
// The serving thread will represent market info by a read write lock protected variable
// The serving thread has two jobs
// - Write to output file for us to analyze later
// - Serve agents who want to read data
// The serving thread will spin, and in each loop non-blocking check if it has new data. If it does, it will write the data to an output file, then acquire the write lock, and then write data to local var, drop the lock, allowing other agents to read the data

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
// Agent sends order to Market
// Market acts on Account and send updates on channels
// Market never waits on agents except for orders
// The Agent listens on channels

#[derive(Debug)]
pub struct Account {
    account_sender: mpsc::Sender<AccountInfo>,
    market_info_sender: crossbeam::channel::Sender<String>,
    solvent: bool, 
    account_info: AccountInfo,
}

impl Account {
    fn notify(&mut self) {
        let _ = self.account_sender.send(self.account_info);
    }
}

pub struct Market {
    order_book: OrderBook,
    order_channel: (mpsc::Sender<MakeOrder>, mpsc::Receiver<MakeOrder>),
    market_info_channel: (
        crossbeam::channel::Sender<String>,
        crossbeam::channel::Receiver<String>,
    ),
    accounts: HashMap<Uuid, Account>,
}
impl Market {
    pub fn new() -> Market {
        Market {
            order_book: OrderBook::default(),
            order_channel: mpsc::channel(),
            market_info_channel: crossbeam::channel::unbounded(),
            accounts: HashMap::new(),
        }
    }
    pub fn create_account(
        &mut self,
        init: AccountInfo,
    ) -> (
        mpsc::Sender<MakeOrder>,
        crossbeam::channel::Receiver<String>,
        mpsc::Receiver<AccountInfo>,
    ) {
        let (account_sender, account_receiver) = mpsc::channel();
        let account = Account {
            account_sender,
            market_info_sender: self.market_info_channel.0.clone(),
            solvent: true,
            account_info: init 
        };
        self.accounts.insert(init.id, account);
        (
            self.order_channel.0.clone(),
            self.market_info_channel.1.clone(),
            account_receiver,
        )
    }
    fn delete_old_orders(&mut self, max_age: f64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        println!("{:}", now - max_age);
        self.order_book.cancel_old_orders(now - max_age);
    }

    pub fn run(&mut self) { 
        loop {
            let received = self.order_channel.1.recv().unwrap();
            match received {
                MakeOrder::Submit(order) => self.handle_order(order), 
                MakeOrder::Cancel(order) => self.cancel_order(order),
                MakeOrder::External(order) => { 
                    if let Ok(order) = Order::build(order) {
                        println!("INCOMING");
                        println!("limit: {:.8?} \t quantity: {} \t id: {}", &order.limit.map(|l| f64::from(l)), &order.quantity, &order.account_id);

                        self.record_order(order) 
                    }
                    self.do_clearing();
                    println!("AFTER CLEARING");
                    println!("{}", &self.order_book);
                    println!("{:#?}", &self.accounts);
                    println!("===")
                }
            }
            self.delete_old_orders(10.);
        }
    }

    fn handle_order(&mut self, order: SubmitOrder) {
        match self.validate_order(order) {
            Ok(order) => self.record_order(order),
            Err(_) => return ()
        };

        // println!("BEFORE CLEARING");
        // println!("{:#?}", &self.order_book);
        // println!("===");
        self.do_clearing();
        // println!("AFTER CLEARING");
        // println!("{:#?}", &self.order_book);
        // println!("{:#?}", &self.accounts);
        // println!("===")
    }
    fn do_clearing(&mut self) {
        loop {
            let Some(best_ask) = self.order_book.peek(Side::Ask) else {
                break
            };
            let Some(best_bid) = self.order_book.peek(Side::Bid) else {
                break
            };

            if best_ask.limit.is_none() || best_bid.limit.is_none() {
                 // If only one is a market order, price is the best limit
                 // If both are market, then look for the best limit price on each side. The older one gets the best fill. If both are same age, then they fill at the midpoint. If there are no best limit prices, then no transaction 
                 todo!();
            } else {
                if best_ask.limit > best_bid.limit {
                    break
                }

                let qty = min(best_ask.quantity, best_bid.quantity);
                let price = match best_ask.timestamp.cmp(&best_bid.timestamp) {
                    Ordering::Less => best_bid.limit.unwrap(),
                    Ordering::Equal => (best_ask.limit.unwrap() + best_bid.limit.unwrap()) / 2.,
                    Ordering::Greater => best_ask.limit.unwrap() 
                    // the older order gets the better fill
                }; 
                
                let Some(buyer) = self.accounts.get(&best_bid.account_id) else { 
                    self.order_book.pop(Side::Bid);
                    break
                };
                let Some(seller) = self.accounts.get(&best_ask.account_id) else {
                    self.order_book.pop(Side::Ask); 
                    break
                };
                
                if let Some(buyer) = self.accounts.get_mut(&best_bid.account_id) { 
                    buyer.account_info.stocks += qty as i32;
                    buyer.account_info.cash -= f64::from(price) * qty as f64;
                    buyer.notify();
                }
                
                if let Some(seller) = self.accounts.get_mut(&best_ask.account_id) { 
                    seller.account_info.stocks -= qty as i32;
                    seller.account_info.cash += f64::from(price) * qty as f64;
                    seller.notify();
                }

                self.order_book.consume(&qty);
            }
        } 
        self.market_info_channel.0.send(
            format!("{:?} | {:?}", self.order_book.peek(Side::Ask), self.order_book.peek(Side::Bid))
        ).unwrap(); 
    }

    fn record_order(&mut self, order: Order) { 
        self.order_book.insert(order);
        // println!("{:#?}", self.order_book);
    }

    fn validate_order(&self, order: SubmitOrder) -> Result<Order, ()> {
        let order = Order::build(order).map_err(|_| ())?;
        if self.order_book.is_wash(&order) {
            return Err(())
        }
        Ok(order)
    }
    fn cancel_order(&mut self, order: CancelOrder) {
        self.order_book.cancel(order.order_id);
    }
}

pub enum MakeOrder {
    Submit(SubmitOrder),
    Cancel(CancelOrder),
    External(SubmitOrder)
}

pub struct SubmitOrder {
    pub limit: Option<f64>, // if limit is None, order is treated as a market order
    pub quantity: usize,
    pub side: Side,
    pub account_id: Uuid, // really should use a constructor or builder rather than making these public
}
pub struct CancelOrder {
    order_id: Uuid,
}

#[derive(Debug)]
struct Order {
    limit: Option<NotNan<f64>>,
    quantity: usize,
    side: Side,
    account_id: Uuid,
    timestamp: NotNan<f64>,
    id: Uuid
}

impl Order {
    fn build(order: SubmitOrder) -> Result<Order, &'static str>{
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "Error in generating timestamp")?.as_secs_f64();
        Ok(Order {
            limit: order.limit.map(|limit| NotNan::new(limit)).transpose().map_err(|_| "Limit must be not Nan")?,
            timestamp: NotNan::new(timestamp).map_err(|_| "Error in generating timestamp")?,
            quantity: order.quantity,
            side: order.side,
            account_id: order.account_id,
            id: Uuid::new_v4(),
        })
    }
}

#[derive(Default, Debug)]
struct OrderBook {
    bids: Vec<BidOrder>,
    asks: Vec<AskOrder>
}

impl Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display = String::new(); 
        display.push_str("BIDS\n");
        display.push_str(&format!("len: {:}\n", self.len(Side::Bid)));
        for order in self.best(Side::Bid, 100).iter().rev() {
            display.push_str(&format!("{:.8?} \t {} \t {} \t {}\n", order.limit.map(|l| f64::from(l)), order.quantity, order.account_id, order.timestamp));
        }
        display.push_str("ASKS\n");
        display.push_str(&format!("len: {:}\n", self.len(Side::Ask)));
        for order in self.best(Side::Ask, 100) {
            display.push_str(&format!("{:.8?} \t {} \t {} \t {}\n", order.limit.map(|l| f64::from(l)), order.quantity, order.account_id, order.timestamp));
        }
        write!(f, "{}", display)
    }
}

impl OrderBook {
    fn insert(&mut self, order: Order) {
        match order.side {
            Side::Bid => {
                self.bids.push(BidOrder{order});
                self.bids.sort();
            }
            Side::Ask => {
                self.asks.push(AskOrder{order});
                self.asks.sort();
            }
        } 
    }
    fn is_empty(&self, side: Side) -> bool {
        match side {
            Side::Bid => {
                self.bids.is_empty()
            }
            Side::Ask => {
                self.asks.is_empty()
            }
        }
    }

    fn is_wash(&self, order: &Order) -> bool {
        match order.side {
            Side::Bid => {
                if let Some(best_ask) = self.asks.iter().filter(|o| o.order.account_id == order.account_id).rev().next() {
                    if best_ask.order.limit.is_none() {
                        todo!();
                        return true // TODO: how to handle validating market orders against wash trades. It's simple actually. J=
    // We can only submit market orders if we don't have any opposing limit orders and we can only submit limit orders if we don't have any opposing market orders. On top of the other rule that we can only submit limit orders if we don't have any are no opposing orders 
                    };
                    if order.limit.is_none() {
                        todo!();
                    };
                    println!("{:?}", best_ask.order);
                    println!("{:?}", order);
                    if order.limit.unwrap() >= best_ask.order.limit.unwrap() {
                        return true
                    };
                };
                return false
            }
            Side::Ask => {
                if let Some(best_bid) = self.bids.iter().filter(|o| o.order.account_id == order.account_id).rev().next() {
                    if best_bid.order.limit.is_none() {
                        todo!();
                        return true
                    }; 
                    if order.limit.is_none() {
                        todo!();
                    };

                    println!("{:?}", best_bid.order);
                    println!("{:?}", order);
                    if order.limit.unwrap() <= best_bid.order.limit.unwrap() {
                        return true
                    };
                };
                return false
            }
        } 
    }
    fn peek(&self, side: Side) -> Option<&Order> {
        match side {
            Side::Bid => Some(&self.bids.last()?.order),
            Side::Ask => Some(&self.asks.last()?.order)
        } 
    }
    
    fn pop(&mut self, side: Side) -> Option<Order> {
        match side {
            Side::Bid => Some(self.bids.pop()?.order),
            Side::Ask => Some(self.asks.pop()?.order)
        }
    }

    fn consume(&mut self, quantity: &usize) {
        self.bids.last_mut().unwrap().order.quantity -= quantity;
        if self.bids.last().unwrap().order.quantity == 0 {
            self.bids.pop();
        } 
        self.asks.last_mut().unwrap().order.quantity -= quantity;
        if self.asks.last().unwrap().order.quantity == 0 {
            self.asks.pop();
        }
    }

    fn cancel(&mut self, order_id: Uuid) -> Option<Order>{
        let mut order = None;
        if let Some(index) = self.bids.iter().position(|o| o.order.id == order_id) {
            order = Some(self.bids.remove(index).order);
        }
        if let Some(index) = self.asks.iter().position(|o| o.order.id == order_id) {
            order = Some(self.asks.remove(index).order);
        }
        order 
    }
    fn cancel_old_orders(&mut self, before: f64) {
        let bid_len = self.bids.len();
        let ask_len = self.asks.len();
        self.bids.retain(|o| f64::from(o.order.timestamp) > before);
        self.asks.retain(|o| f64::from(o.order.timestamp) > before);
    }

    fn best(&self, side: Side, n: usize) -> Vec<&Order> {
        match side {
            Side::Bid => self.bids.iter().map(|o| &o.order).rev().take(n).collect(),
            Side::Ask => self.asks.iter().map(|o| &o.order).rev().take(n).collect()
        } 
    }
    
    fn len(&self, side: Side) -> usize {
        match side {
            Side::Bid => self.bids.len(),
            Side::Ask => self.asks.len()
        } 
    }
}

#[derive(Debug)]
struct AskOrder {
    order: Order,
}

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.order.limit.is_none() && other.order.limit.is_none() {
            return other.order.timestamp.cmp(&self.order.timestamp); 
        }
        else {
            if self.order.limit.is_none() {
                return Ordering::Greater;
            }
            if other.order.limit.is_none() {
                return Ordering::Less;
            }
        }
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
    order: Order,
}

impl Ord for BidOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.order.limit.is_none() && other.order.limit.is_none() {
            return other.order.timestamp.cmp(&self.order.timestamp); 
        }
        else {
            if self.order.limit.is_none() {
                return Ordering::Greater;
            }
            if other.order.limit.is_none() {
                return Ordering::Less;
            }
        }
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

#[derive(Clone, Copy, Debug)]
pub struct AccountInfo {
    pub cash: f64,
    pub stocks: i32,
    pub id: Uuid,
}

pub struct Agent {
    account_receiver: mpsc::Receiver<AccountInfo>,
    market_info_receiver: crossbeam::channel::Receiver<String>,
    order_sender: mpsc::Sender<MakeOrder>,
    pub account_info: AccountInfo,
}
impl Agent {
    pub fn new(cash: f64, stocks: i32, market: &mut Market) -> Agent {
        let init = AccountInfo {
            cash,
            stocks,
            id: Uuid::new_v4(),
        };
        let (order_sender, market_info_receiver, account_receiver) = market.create_account(init);
        Agent {
            account_receiver,
            market_info_receiver,
            order_sender,
            account_info: init,
        }
    }
    pub fn test_run(&self, mut order_sequence: Vec<(SubmitOrder, u64)>) {
        let mut i = 0;
        loop {
            let market_info = self.market_info_receiver.try_recv();

            // println!("{:#?}", market_info);
            if order_sequence.len() > 0 { 
                let (order, delay) = order_sequence.pop().unwrap();

                thread::sleep(Duration::from_millis(delay));

                println!("{}, {}", self.account_info.id, i);
                let _ =self.order_sender.send(MakeOrder::Submit(
                    order
                            ));
                i += 1;
            }
        }
    }
}

pub struct Broker { 
    // represents sum total of external order flow modelled as a Poisson process
    account_receiver: mpsc::Receiver<AccountInfo>,
    order_sender: mpsc::Sender<MakeOrder>,
    pub account_info: AccountInfo,

    
    lambda_s: f64, 
    lambda_c: f64,
    sigma_q: f64,
    sigma_p: f64,
    p_f: f64, // fundamental price
    
    
}

impl Broker {
    pub fn build(lambda_s: f64, lambda_c: f64, sigma_p: f64, sigma_q: f64, p_f: f64, market: &mut Market) -> Result<Broker, &'static str> {
        if lambda_c <= 0. || lambda_s <= 0. || sigma_p <= 0. || p_f <= 0. || sigma_q <= 0. {
            return Err("Please provide positive parameters")
        };
        let init = AccountInfo {
            cash: 0.,
            stocks: 0,
            id: Uuid::new_v4()
        };
        let (order_sender, _, account_receiver) = market.create_account(init);
        Ok(Broker {
            account_receiver,
            order_sender,
            account_info: init,
            lambda_s,
            lambda_c,
            sigma_p,
            sigma_q,
            p_f
        })
    }
    pub fn run(&mut self) {
        let exp_dist_s = Exp::new(self.lambda_s).unwrap();
        let exp_dist_c = Exp::new(self.lambda_c).unwrap();
        let logn_dist = LogNormal::new(0., self.sigma_p).unwrap();
        let norm_dist = Normal::new(0., self.sigma_q).unwrap();
        let bern_dist = Bernoulli::new(0.5).unwrap();
       
        let share = Arc::new(Mutex::new(self));
        let submit_self = share.clone();
        let cancel_self = share.clone();
        let submit_thread = crossbeam::thread::scope(move |_| { 
            let mut rng = rand::thread_rng();
            let self_ = submit_self; 
            loop { 
                let mut self_ = self_.lock().unwrap();
                if let Ok(account_info) = self_.account_receiver.try_recv() {
                    self_.account_info = account_info; 
                }
                let _ = self_.order_sender.send(MakeOrder::External(
                    SubmitOrder {
                        account_id: self_.account_info.id,
                        side: if bern_dist.sample(&mut rng) {Side::Ask} else {Side::Bid},
                        limit: Some(logn_dist.sample(&mut rng) * self_.p_f),
                        quantity: max(1, norm_dist.sample(&mut rng).abs().trunc() as usize)
                    }
                ));
                drop(self_);
                thread::sleep(Duration::from_secs_f64(exp_dist_s.sample(&mut rng)));
            } 
        });
        // let cancel_thread = crossbeam::thread::scope(move |_| {
        //     let mut rng = rand::thread_rng();
        //     let self_ = cancel_self;
        //     loop {
        //         
        //         thread::sleep(Duration::from_secs_f64(exp_dist_c.sample(&mut rng)));
        //     }
        // });
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // #[test]
    fn simple_sequence() {
    
        let mut market = Market::new();
        let agent1 = Agent::new(10000., 0, &mut market);
        let agent2 = Agent::new(2000., 4, &mut market);

        let agent1_handler = thread::spawn(move || {
            let mut order_sequence = vec![
                (SubmitOrder {
                    limit: Some(20.), quantity: 10, side: Side::Ask, account_id: agent1.account_info.id,
                }, 500),
                (SubmitOrder {
                    limit: Some(30.), quantity: 20, side: Side::Ask, account_id: agent1.account_info.id,
                }, 500),
                (SubmitOrder {
                    limit: Some(15.), quantity: 1, side: Side::Ask, account_id: agent1.account_info.id,
                }, 500),
                (SubmitOrder {
                    limit: Some(20.), quantity: 30, side: Side::Ask, account_id: agent1.account_info.id,
                }, 500),
                ];
            order_sequence.reverse();
            agent1.test_run(order_sequence);
        });
        let agent2_handler = thread::spawn(move || { 
            println!("hello");
            let order_sequence = vec![
                (SubmitOrder {
                    limit: Some(21.), quantity: 23, side: Side::Bid, account_id: agent2.account_info.id
                }, 3000)
            ];
            agent2.test_run(order_sequence);
        });

        let market_handler = thread::spawn(move || {
            market.run();
        });

        thread::spawn(|| {thread::sleep(Duration::from_millis(10000));}).join();
         
        println!("helloworld");

        // Kinda janky run with `cargo test -- --nocapture`
        // But I expect we will end up rewriting agents so much that this test shouldn't last for too long
        // TODO: Make Agent a trait and in the test cases, move the current agent definition in as an impl of that trait. Then run this. Also remove the janky nocapture and have proper asserts
    }

    #[test]
    fn more_complex_sequence() { 
        let mut market = Market::new();
        let alice = Agent::new(1e5, 0, &mut market);
        let bob = Agent::new(1e5, 0, &mut market);
        let charlie = Agent::new(1e5, 1000, &mut market);
        let dan = Agent::new(1e5, 1000, &mut market);

        println!("alice: {}", alice.account_info.id);
        println!("bob: {}", bob.account_info.id);
        println!("charlie: {}", charlie.account_info.id);
        println!("dan: {}", dan.account_info.id);

        let alice_handler = thread::spawn(move || {
            let mut order_sequence = vec![
                (SubmitOrder {
                    limit: Some(60.01), quantity: 30, side: Side::Bid, account_id: alice.account_info.id,
                }, 100),
                (SubmitOrder {
                    limit: Some(60.11), quantity: 12, side: Side::Ask, account_id: alice.account_info.id,
                }, 100),
                (SubmitOrder {
                    limit: Some(60.02), quantity: 15, side: Side::Bid, account_id: alice.account_info.id,
                }, 1500),
                (SubmitOrder {
                    limit: Some(60.08), quantity: 14, side: Side::Ask, account_id: alice.account_info.id,
                }, 100),
                (SubmitOrder {
                    limit: Some(60.08), quantity: 8, side: Side::Ask, account_id: alice.account_info.id,
                }, 2000)
                ];
            order_sequence.reverse();
            alice.test_run(order_sequence);
        });
        let bob_handler = thread::spawn(move || { 
            let mut order_sequence = vec![
                (SubmitOrder {
                    limit: Some(60.08), quantity: 100, side: Side::Bid, account_id: bob.account_info.id
                }, 300),
                (SubmitOrder {
                    limit: Some(60.20), quantity: 10, side: Side::Ask, account_id: bob.account_info.id
                }, 100),
            ];
            order_sequence.reverse();
            
            thread::sleep(Duration::from_millis(900));
            bob.test_run(order_sequence);
        });
        let charlie_handler = thread::spawn(move || {
            thread::sleep(Duration::from_millis(2133));
            let mut order_sequence = vec![
                (SubmitOrder {
                    limit: Some(60.01), quantity: 120, side: Side::Ask, account_id: charlie.account_info.id
                }, 500)
            ];
            charlie.test_run(order_sequence);
        });
        let dan_handler = thread::spawn(move || {
            thread::sleep(Duration::from_millis(2266));
            let mut order_sequence = vec![
                (SubmitOrder {
                    limit: Some(60.11), quantity: 20, side: Side::Bid, account_id: dan.account_info.id
                }, 1000),
                (SubmitOrder {
                    limit: Some(60.3), quantity: 10, side: Side::Ask, account_id: dan.account_info.id
                }, 100)
            ];
            order_sequence.reverse();
            dan.test_run(order_sequence);
            
        });


        let market_handler = thread::spawn(move || {
            market.run();
        });

        thread::spawn(|| {thread::sleep(Duration::from_millis(10000));}).join();
         
        println!("helloworld");

    }
}
