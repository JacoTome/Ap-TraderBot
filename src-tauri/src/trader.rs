use std::collections::HashMap;
use std::sync::Arc;

use bose::market::BoseMarket;
use rcnz_market::rcnz::RCNZ;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use tokio::sync::mpsc;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;
use BVC::BVCMarket;

use crate::TraderState;
pub struct Trader {
    name: String,
    goods: HashMap<GoodKind, f32>,
    earnings: f32,
    budget: f32,
}

impl Trader {
    pub fn new(name: String) -> Self {
        let mut trader = Trader {
            name: name,
            goods: HashMap::new(),
            earnings: 0.0,
            budget: 10000.0,
            // markets: Vec::new(),
        };
        trader.goods.insert(GoodKind::EUR, 1000.0);
        trader.goods.insert(GoodKind::USD, 1000.0);
        trader.goods.insert(GoodKind::YUAN, 1000.0);
        trader.goods.insert(GoodKind::YEN, 1000.0);

        trader
    }
    pub fn get_name(&self) -> &str {
        &self.name.as_str()
    }

    pub fn get_trader_state(&self) -> TraderState {
        let random_earning = rand::random::<f32>() * 100.0;
        let random_budget = rand::random::<f32>() * 100.0;
        TraderState {
            name: self.name.clone(),
            goods: self.goods.clone(),
            earnings: random_earning,
            budget: random_budget,
        }
    }
}

pub async fn run(mut input_rx: mpsc::Receiver<String>, output_tx: mpsc::Sender<TraderState>) {
    let trader = Trader::new("Trader 1".to_string());

    loop {
        if input_rx.recv().await.is_some() {
            let output = trader.get_trader_state();
            output_tx.send(output).await.expect("send failed");
        }
    }
}

// pub async fn async_process_model(
//     mut input_rx: mpsc::Receiver<String>,
//     output_tx: mpsc::Sender<String>,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     // send trader name
//     let trader = Trader::new("Trader 1".to_string());

//     while let Some(input) = input_rx.recv().await {
//         let mut output = input;
//         output.push_str(&trader.get_name());
//         println!("output: {}", output);
//         output_tx.send(output).await?;
//     }

//     Ok(())
// }
