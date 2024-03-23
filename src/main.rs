use std::thread;

use market_simulation_reattempt::{Market, Agent, Broker, Side, MakeOrder, SubmitOrder, CancelOrder};

fn main() {
//     let mut market = Market::new();
//     let agent1 = Agent::new(10000., 0, &mut market);
//     let agent2 = Agent::new(2000., 4, &mut market);
    let mut market = Market::new();
    let mut broker = Broker::build(20., 20., 0.05, 100., 100., &mut market).unwrap();

    let broker_handler = thread::spawn(move || {
        broker.run();
    });

    let market_handler = thread::spawn(move || {
        market.run();
    });

// 
//     let agent1_handler = thread::spawn(move || {
//         let mut order_sequence = vec![
//             SubmitOrder {
//                 limit: Some(20.), quantity: 10, side: Side::Ask, account_id: agent1.account_info.id,
//             },
//             SubmitOrder {
//                 limit: Some(30.), quantity: 20, side: Side::Ask, account_id: agent1.account_info.id,
//             },
//             SubmitOrder {
//                 limit: Some(15.), quantity: 1, side: Side::Ask, account_id: agent1.account_info.id,
//             },
//             SubmitOrder {
//                 limit: Some(20.), quantity: 30, side: Side::Ask, account_id: agent1.account_info.id,
//             },
//             ];
//         order_sequence.reverse();
//         agent1.run(order_sequence);
//     });
//     let agent2_handler = thread::spawn(move || { 
//         thread::sleep(Duration::from_millis(3000));
//         println!("hello");
//         let order_sequence = vec![
//             SubmitOrder {
//                 limit: Some(21.), quantity: 23, side: Side::Bid, account_id: agent2.account_info.id
//             }
//         ];
//         agent2.run(order_sequence);
//     });
// 
//     let market_handler = thread::spawn(move || {
//         market.run();
//     });
// 
    market_handler.join();
    println!("helloworld");
}
