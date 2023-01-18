extern crate BVC;
extern crate bose;
extern crate rcnz_market;
use bose::market::BoseMarket;
use rcnz_market::rcnz::RCNZ;
use unitn_market_2022::{good::good_kind::GoodKind, market::Market};
use BVC::BVCMarket;

pub struct Trader {
    name: String,
    // goods: HashMap<GoodKind, f32>,
    earnings: f32,
    budget: f32,
}
#[derive(Debug, Clone)]
pub struct TraderState {
    name: String,
    earnings: f32,
    budget: f32,
}

impl Trader {
    pub fn new(name: String) -> Self {
        let mut trader = Trader {
            name: name,
            earnings: 0.0,
            budget: 10000.0,
        };
        trader
    }
    pub fn get_name(&self) -> &str {
        &self.name.as_str()
    }
    pub fn get_state(&self) -> TraderState {
        TraderState {
            name: self.name.clone(),
            earnings: self.earnings,
            budget: self.budget,
        }
    }
}

pub fn main() {
    let trader = Trader::new("JTrader".to_string());
    // create Markets
    let bose = BoseMarket::new_random();
    let bvc = BVCMarket::new_random();
    let rcnz = RCNZ::new_random();
    // print traderState

    // get goods from markets
    let bose_goods = bose.borrow().get_goods();
    let bvc_goods = bvc.borrow().get_goods();
    let rcnz_goods = rcnz.borrow().get_goods();

    let markets_goods = vec![bose_goods.clone(), bvc_goods.clone(), rcnz_goods.clone()];

    // find best exchange rate for each good
    let mut best_exchange_rate_buy = vec![0.0, 0.0, 0.0, 0.0];
    let mut best_exchange_rate_sell = vec![0.0, 0.0, 0.0, 0.0];
    for i in 0..4 {
        for goods in &markets_goods {
            let exchange_rate_buy = goods[i].exchange_rate_buy;
            let exchange_rate_sell = goods[i].exchange_rate_sell;
            if exchange_rate_buy > best_exchange_rate_buy[i] {
                best_exchange_rate_buy[i] = exchange_rate_buy;
            }
            if exchange_rate_sell > best_exchange_rate_sell[i] {
                best_exchange_rate_sell[i] = exchange_rate_sell;
            }
        }
    }

    // print best exchange rates
    for _i in 0..10 {
        println!("TraderState: {:?}", trader.get_state());
        // wait 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
