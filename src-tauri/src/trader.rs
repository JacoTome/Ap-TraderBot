use std::collections::HashMap;
use tauri::{App, Manager, Window};

use unitn_market_2022::good::good_kind::GoodKind;
#[derive(Debug)]
pub struct Trader {
    name: String,
    goods: HashMap<GoodKind, f32>,
    // reference to the application
    app: Window,
}

impl Trader {
    pub fn new(name: String, app: Window) -> Self {
        let mut trader = Trader {
            name: name,
            app: app,
            goods: HashMap::new(),
        };
        trader.goods.insert(GoodKind::EUR, 1000.0);
        trader.goods.insert(GoodKind::USD, 1000.0);
        trader.goods.insert(GoodKind::YUAN, 1000.0);
        trader.goods.insert(GoodKind::YEN, 1000.0);

        trader
    }

    pub fn get_name(&self) -> String {
        self.app
            .emit("trader-name", self.name.clone())
            .expect("error while emitting event");
        self.name.clone()
    }
    pub fn run() {}
}
