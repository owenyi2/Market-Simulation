use std::thread;
use rayon::prelude::*;

use market_simulation_reattempt::{Market, Agent, Broker, Side, MakeOrder, SubmitOrder, CancelOrder};

fn main() {
//     let mut market = Market::new();
//     let agent1 = Agent::new(10000., 0, &mut market);
//     let agent2 = Agent::new(2000., 4, &mut market);
    let mut market = Market::new();
    let mut broker = Broker::build(10., 20., 0.05, 10., 100., &mut market).unwrap();

    let mut agents = Vec::new();
    for i in 0..4 {
        agents.push(Agent::new(10000., 0, &mut market));
    }
    let broker_handler = thread::spawn(move || {
        broker.run();
    });

    let market_handler = thread::spawn(move || {
        market.run();
    });

    for mut agent in agents.into_iter() {
        thread::spawn(move || {
            agent.run();
        });
    } 
    market_handler.join();
    println!("helloworld");
}
