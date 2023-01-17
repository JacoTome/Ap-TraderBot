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
    let trader = Trader::new("michele".to_string());
    // create Markets
    let bose = BoseMarket::new_random();
    let bvc = BVCMarket::new_random();
    let rcnz = RCNZ::new_random();
    // print traderState
    let state = trader.get_state();

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

    // print markets goods
    println!("Bose goods:");
    for good in &bose_goods {
        println!("{:?}", good);
    }
    println!("BVC goods:");
    for good in &bvc_goods {
        println!("{:?}", good);
    }
    println!("RCNZ goods:");
    for good in &rcnz_goods {
        println!("{:?}", good);
    }

    // print best exchange rates
    println!("Best exchange rates:");
    for i in 0..4 {
        println!(
            "Good: {} | Buy: {} | Sell: {}",
            bose_goods[i].good_kind.to_string(),
            best_exchange_rate_buy[i],
            best_exchange_rate_sell[i]
        );
    }
}
