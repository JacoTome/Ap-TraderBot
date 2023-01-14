use std::collections::HashMap;

use tokio::sync::mpsc;
use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug)]
pub struct Trader {
    name: String,
    goods: HashMap<GoodKind, f32>,
    // reference to the application
}

impl Trader {
    pub fn new(name: String) -> Self {
        let mut trader = Trader {
            name: name,
            goods: HashMap::new(),
        };
        trader.goods.insert(GoodKind::EUR, 1000.0);
        trader.goods.insert(GoodKind::USD, 1000.0);
        trader.goods.insert(GoodKind::YUAN, 1000.0);
        trader.goods.insert(GoodKind::YEN, 1000.0);

        trader
    }
}

pub async fn async_process_model(
    mut input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // send trader name
    let trader = Trader::new("Trader 1".to_string());
    while let Some(input) = input_rx.recv().await {
        let mut output = input;
        output.push_str(&trader.name);
        println!("output: {}", output);
        output_tx.send(output).await?;
    }

    Ok(())
}
