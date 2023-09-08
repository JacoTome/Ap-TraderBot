/*
Obbiettivo: creare un trader che parta con x euro e ne guadagni il più possibile in un tempo limitato (y giorni)
 */

use crate::trader::trader_ricca::consts::STARTING_EUR;
mod market_calculations;
mod market_intetactions;
mod market_strategies;

extern crate core;

use crate::utils::market::{CurrencyData, DailyData, MarketData, MarketEvent};

use crate::utils::market::TraderTrait;

use std::cell::RefCell;

use std::collections::HashMap;
use std::rc::Rc;
use unitn_market_2022;
use unitn_market_2022::good::good_kind::GoodKind;

use unitn_market_2022::market::Market;

use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::{subscribe_each_other, wait_one_day};

use bose::market::BoseMarket;
use BVC::BVCMarket;

pub struct Trade {
    pub trans_kind: unitn_market_2022::event::event::EventKind,
    pub kind_giving: GoodKind,
    pub kind_receiving: GoodKind,
    pub giving_quantity: f32,
    pub receiving_quantity: f32,
    pub market_index: usize,
}

pub struct GoodMetadata {
    pub good: unitn_market_2022::good::good::Good,
    pub exchange_rate: f32,
}

pub struct TraderRicca {
    markets: Vec<Rc<RefCell<dyn Market>>>,
    goods: HashMap<GoodKind, GoodMetadata>,
    traders_locked: HashMap<String, Trade>,
    day: i32,
    name: String,
    not_optimal_counter: i32,
    last_daily_data: Vec<DailyData>,
    last_market_data: Vec<Vec<MarketData>>,
    done: bool,
    end_game_stuck: i32,
}

pub fn get_default_exchange(good_kind: GoodKind) -> f32 {
    match good_kind {
        GoodKind::EUR => 1.,
        GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
        GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
        GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    }
}

impl TraderTrait for TraderRicca {
    fn initialize_trader() -> Self {
        return TraderRicca::init_trader(TraderRicca::init_markets());
    }

    fn progess_day(&mut self, strat_index: i32) {
        self.last_daily_data = Vec::new();
        self.last_market_data = Vec::new();

        match strat_index {
            0 => self.main_strat_day(),
            1 => self.buy_all_of_kind_strat_day(),
            2 => self.random_buy_strat_day(),
            3 => self.buy_low_sell_high_strat_day(),
            _ => self.main_strat_day(),
        }
    }

    fn get_daily_data(&self) -> Vec<DailyData> {
        self.last_daily_data.clone()
    }

    fn get_market_data(&self) -> Vec<Vec<MarketData>> {
        return self.last_market_data.clone();
    }

    fn get_trader_data(&self) -> CurrencyData {
        let currency_data = CurrencyData {
            eur: self.goods.get(&GoodKind::EUR).unwrap().good.get_qty() as f64,
            usd: self.goods.get(&GoodKind::USD).unwrap().good.get_qty() as f64,
            yen: self.goods.get(&GoodKind::YEN).unwrap().good.get_qty() as f64,
            yuan: self.goods.get(&GoodKind::YUAN).unwrap().good.get_qty() as f64,
        };

        return currency_data;
    }
}

impl TraderRicca {
    pub fn add_market_event(&mut self) {
        let mut data: Vec<MarketData> = Vec::new();

        self.markets.iter().enumerate().for_each(|(_, market)| {
            let market = market.borrow();
            let mut currency_data = CurrencyData {
                eur: 0.,
                usd: 0.,
                yen: 0.,
                yuan: 0.,
            };

            market.get_goods().iter().for_each(|g| match g.good_kind {
                GoodKind::EUR => currency_data.eur = g.quantity as f64,
                GoodKind::USD => currency_data.usd = g.quantity as f64,
                GoodKind::YEN => currency_data.yen = g.quantity as f64,
                GoodKind::YUAN => currency_data.yuan = g.quantity as f64,
            });

            data.push(MarketData {
                currencies: currency_data,
                name: market.get_name().to_string(),
            });
        });

        self.last_market_data.push(data);
    }

    pub fn init_markets() -> Vec<Rc<RefCell<dyn Market>>> {
        let mut markets: Vec<Rc<RefCell<dyn Market>>> = Vec::new();

        markets.push(BVCMarket::new_random());
        markets.push(BoseMarket::new_random());
        markets.push(rcnz_market::rcnz::RCNZ::new_random());

        /*
        //print markets goods
        markets.iter().for_each(|market| {
            let market = market.borrow();
            println!("{}: {:?}", market.get_name(), market.get_goods());
        });
         */

        //markets.push(RCNZ::new_random());

        subscribe_each_other!(markets[0], markets[1], markets[2]);
        //subscribe_each_other!(markets[0], markets[1]);

        markets
    }

    pub fn init_trader(markets: Vec<Rc<RefCell<dyn Market>>>) -> TraderRicca {
        let mut trader = TraderRicca {
            markets: markets,
            goods: HashMap::new(),
            traders_locked: HashMap::new(),
            day: 0,
            name: String::from("Emanuele"),
            end_game_stuck: 0,
            not_optimal_counter: 0,
            last_daily_data: Vec::new(),
            last_market_data: Vec::new(),
            done: false,
        };

        trader.add_market_event();

        trader.goods.insert(
            GoodKind::EUR,
            GoodMetadata {
                good: Good::new(GoodKind::EUR, STARTING_EUR),
                exchange_rate: 1.0,
            },
        );
        trader.goods.insert(
            GoodKind::USD,
            GoodMetadata {
                good: Good::new(GoodKind::USD, 0.),
                exchange_rate: DEFAULT_EUR_USD_EXCHANGE_RATE,
            },
        );
        trader.goods.insert(
            GoodKind::YUAN,
            GoodMetadata {
                good: Good::new(GoodKind::YUAN, 0.),
                exchange_rate: DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            },
        );
        trader.goods.insert(
            GoodKind::YEN,
            GoodMetadata {
                good: Good::new(GoodKind::YEN, 0.),
                exchange_rate: DEFAULT_EUR_YEN_EXCHANGE_RATE,
            },
        );

        trader
    }

    pub fn wait_update(&mut self) {
        wait_one_day!(self.markets[0], self.markets[1]);
        self.last_daily_data.push(DailyData {
            event: MarketEvent::Wait,
            amount_given: 0.,
            amount_received: 0.,
            kind_given: GoodKind::EUR,
            kind_received: GoodKind::EUR,
        });
        self.add_market_event();
        self.day_passed();
    }

    pub fn day_passed(&mut self) {
        self.day += 1;
    }
}
