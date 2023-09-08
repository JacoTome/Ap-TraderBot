use crate::utils::market::{CurrencyData, DailyData, MarketData, MarketEvent};

use crate::utils::market::TraderTrait;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use unitn_market_2022;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;

use bose::market::BoseMarket;
use rand::{thread_rng, Rng};
use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use BVC::BVCMarket;

const STARTING: f32 = 10000.0;
const MAX_DAYS: i32 = 1000;
const START_MIN: f32 = 1000.0;
const START_MAX: f32 = 3000.0;
const START_PERCENTAGE: f32 = 2.0;

pub struct Trade {
    pub trans_kind: unitn_market_2022::event::event::EventKind,
    pub kind_giving: GoodKind,
    pub kind_receiving: GoodKind,
    pub giving_quantity: f32,
    pub receiving_quantity: f32,
    pub market_index: usize,
}

#[derive(Debug)]
pub struct GoodMetadata {
    pub good: unitn_market_2022::good::good::Good,
    pub exchange_rate: f32,
}

pub struct TraderMaccacaro {
    markets: Vec<Rc<RefCell<dyn Market>>>,
    goods: HashMap<GoodKind, GoodMetadata>,
    day: i32,
    _name: String,
    last_daily_data: Vec<DailyData>,
    last_market_data: Vec<Vec<MarketData>>,
    min: f32,
    max: f32,
    percentage: f32,
    sell_locked: Vec<(GoodKind, f32, usize, f32, String)>,
    buy_locked: Vec<(GoodKind, f32, usize, f32, String)>,
    done: bool,
    sell: bool,
    buy: (bool, GoodKind),
}

impl TraderTrait for TraderMaccacaro {
    fn initialize_trader() -> Self {
        return TraderMaccacaro::init_trader(TraderMaccacaro::init_markets());
    }
    fn progress_day(&mut self, _strat_index: i32) {
        self.last_daily_data = Vec::new();
        self.last_market_data = Vec::new();
        self.default();
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

impl TraderMaccacaro {
    pub fn add_market_event(&mut self) {
        let mut data: Vec<MarketData> = Vec::new();

        self.markets.iter().enumerate().for_each(|(_i, market)| {
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
        // markets.push((RCNZ::new_random()));

        // subscribe_each_other!(markets[0], markets[1], markets[2]);
        subscribe_each_other!(markets[0], markets[1]);
        markets
    }

    pub fn init_trader(markets: Vec<Rc<RefCell<dyn Market>>>) -> TraderMaccacaro {
        let mut trader = TraderMaccacaro {
            markets: markets,
            goods: HashMap::new(),
            day: 0,
            _name: String::from("Maccacaro"),
            last_daily_data: Vec::new(),
            last_market_data: Vec::new(),
            min: START_MIN,
            max: START_MAX,
            percentage: START_PERCENTAGE,
            sell_locked: Vec::new(),
            buy_locked: Vec::new(),
            done: false,
            sell: false,
            buy: (false, GoodKind::EUR),
        };

        trader.add_market_event();

        trader.goods.insert(
            GoodKind::EUR,
            GoodMetadata {
                good: Good::new(GoodKind::EUR, STARTING),
                exchange_rate: 1.0,
            },
        );
        trader.goods.insert(
            GoodKind::USD,
            GoodMetadata {
                good: Good::new(GoodKind::USD, STARTING),
                exchange_rate: DEFAULT_EUR_USD_EXCHANGE_RATE,
            },
        );
        trader.goods.insert(
            GoodKind::YUAN,
            GoodMetadata {
                good: Good::new(GoodKind::YUAN, STARTING),
                exchange_rate: DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            },
        );
        trader.goods.insert(
            GoodKind::YEN,
            GoodMetadata {
                good: Good::new(GoodKind::YEN, STARTING),
                exchange_rate: DEFAULT_EUR_YEN_EXCHANGE_RATE,
            },
        );

        trader
    }

    pub fn set_good_quantity(&mut self, kind: GoodKind, quantity: f32) {
        self.goods.insert(
            kind,
            GoodMetadata {
                good: Good::new(kind, quantity),
                exchange_rate: DEFAULT_EUR_USD_EXCHANGE_RATE,
            },
        );
    }

    pub fn get_good_quantity(&self, kind: GoodKind) -> f32 {
        return self.goods.get(&kind).unwrap().good.get_qty();
    }

    pub fn day(&mut self) {
        self.day += 1;
    }

    pub fn default(&mut self) {
        if self.day < MAX_DAYS {
            let good_kinds = [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
            let mut rng = thread_rng();
            let mut max_buy_price = (GoodKind::EUR, 0., 0, 0.);
            let mut max_sell_price = (GoodKind::EUR, 0., 0, 0.);
            let good;

            let quantity1 = rng.gen_range(self.min..=self.max);
            if self.buy.0 {
                good = self.buy.1;
            } else {
                let good1 = rng.gen_range(0..=3);
                match good1 {
                    0 => good = GoodKind::EUR,
                    1 => good = GoodKind::USD,
                    2 => good = GoodKind::YEN,
                    3 => good = GoodKind::YUAN,
                    _ => good = GoodKind::EUR,
                }
            }

            for i in 0..self.markets.len() {
                let buy_price;
                let sell_price;
                let market = self.markets[i].clone();
                buy_price = match market.borrow().get_buy_price(good, quantity1) {
                    Ok(price) => price,
                    Err(_) => 0.,
                };
                sell_price = match market.borrow().get_sell_price(good, quantity1) {
                    Ok(price) => price,
                    Err(_) => 0.,
                };
                if (buy_price < max_buy_price.1 || max_buy_price.1 == 0.) && buy_price > 0. {
                    max_buy_price = (good, buy_price, i, quantity1);
                }
                if (sell_price > max_sell_price.1 || max_sell_price.1 == 0.) && sell_price > 0. {
                    max_sell_price = (good, sell_price, i, quantity1);
                }
            }

            if self.sell_locked.is_empty() && self.buy_locked.is_empty() {
                let token;

                let rand = rng.gen_range(0..=3);

                if rand == 0 || self.sell && !self.buy.0 {
                    token = self.trader_lock_sell((
                        max_sell_price.0,
                        max_sell_price.1,
                        max_sell_price.2,
                        max_sell_price.3,
                    ));
                    match token {
                        Ok(token) => {
                            self.sell_locked.push((
                                max_sell_price.0,
                                max_sell_price.1,
                                max_sell_price.2 as usize,
                                max_sell_price.3,
                                token,
                            ));
                        }
                        Err(_) => {}
                    }
                } else if rand == 1 || self.buy.0 && !self.sell {
                    token = self.trader_lock_buy((
                        max_buy_price.0,
                        max_buy_price.1,
                        max_buy_price.2,
                        max_buy_price.3,
                    ));
                    match token {
                        Ok(token) => {
                            self.buy_locked.push((
                                max_buy_price.0,
                                max_buy_price.1,
                                max_buy_price.2 as usize,
                                max_buy_price.3,
                                token,
                            ));
                        }
                        Err(_) => {}
                    }
                } else {
                    self.trader_wait_one_day(self.markets.clone());
                }
            } else {
                let buy_locked = self.buy_locked.clone();
                let sell_locked = self.sell_locked.clone();
                if !buy_locked.is_empty() {
                    println!("buy");
                    let last_buy = buy_locked.last().unwrap();
                    if last_buy.1 < self.get_good_quantity(GoodKind::EUR) {
                        let res = self.trader_buy(
                            &last_buy.4,
                            (last_buy.0, last_buy.1, last_buy.2, last_buy.3),
                        );
                        match res {
                            Ok(_) => {
                                self.buy_locked.pop();
                            }
                            Err(_) => {
                                self.buy_locked.pop();
                            }
                        }
                    } else {
                        self.buy_locked.pop();
                        println!("Not enough quantity")
                    }
                } else if !sell_locked.is_empty() {
                    println!("sell");
                    let last_sell = sell_locked.last().unwrap();
                    if last_sell.1 < self.get_good_quantity(last_sell.0) {
                        let res = self.trader_sell(
                            &last_sell.4,
                            (last_sell.0, last_sell.1, last_sell.2, last_sell.3),
                        );
                        match res {
                            Ok(_) => {
                                self.sell_locked.pop();
                            }
                            Err(_) => {
                                self.sell_locked.pop();
                            }
                        }
                    } else {
                        self.sell_locked.pop();
                        println!("Not enough quantity")
                    }
                } else {
                    self.trader_wait_one_day(self.markets.clone());
                }
            }

            let mut upgrade = true;
            for good in good_kinds.iter() {
                if self.get_good_quantity(*good) > STARTING + (STARTING * self.percentage / 100.)
                    && upgrade
                {
                    upgrade = true;
                } else {
                    upgrade = false;
                }
            }
            if upgrade {
                self.min = self.min + (self.min * 0.5);
                self.max = self.max + (self.max * 0.5);
                self.percentage += 2.0;
            }

            self.buy = (false, GoodKind::EUR);
            self.sell = false;
            for good in good_kinds.iter() {
                if self.get_good_quantity(*good) < self.min && !self.buy.0 && !self.sell {
                    if good == &GoodKind::EUR {
                        self.sell = true;
                    } else {
                        self.buy = (true, *good);
                    }
                }
            }
            self.add_market_event();
        } else {
            self.done = true;
        }
    }

    pub fn trader_lock_buy(
        &mut self,
        max_buy_price: (GoodKind, f32, usize, f32),
    ) -> Result<String, ()> {
        println!("\n-----------------\nLock Buy...\n------------------");
        self.day();
        let token = self.markets[max_buy_price.2]
            .as_ref()
            .borrow_mut()
            .lock_buy(
                max_buy_price.0,
                max_buy_price.3,
                max_buy_price.1,
                String::from("Maccacaro"),
            );
        match token {
            Ok(token) => {
                self.last_daily_data.push(DailyData {
                    event: MarketEvent::LockBuy,
                    amount_given: max_buy_price.1 as f64,
                    amount_received: max_buy_price.3 as f64,
                    kind_given: GoodKind::EUR,
                    kind_received: max_buy_price.0,
                });

                self.add_market_event();

                println!("Buy Locked");
                return Ok(token);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(());
            }
        }
    }

    pub fn trader_buy(
        &mut self,
        token: &String,
        max_buy_price: (GoodKind, f32, usize, f32),
    ) -> Result<(), ()> {
        let good = unitn_market_2022::good::good::Good::new(GoodKind::EUR, max_buy_price.1);
        println!("-----------------\nBuy...\n------------------");
        self.day();
        let res = self.markets[max_buy_price.2]
            .as_ref()
            .borrow_mut()
            .buy(token.clone(), &mut good.clone());
        match res {
            Ok(good_bought) => {
                self.last_daily_data.push(DailyData {
                    event: MarketEvent::Buy,
                    amount_given: max_buy_price.1 as f64,
                    amount_received: max_buy_price.3 as f64,
                    kind_given: GoodKind::EUR,
                    kind_received: max_buy_price.0,
                });

                self.add_market_event();

                self.set_good_quantity(
                    good.get_kind(),
                    self.get_good_quantity(good.get_kind()) - good.get_qty(),
                );
                self.set_good_quantity(
                    good_bought.get_kind(),
                    self.get_good_quantity(good_bought.get_kind()) + good_bought.get_qty(),
                );
                println!("Good Bought: {:?}, for {:?}", good_bought, good.clone());
                return Ok(());
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(());
            }
        }
    }

    pub fn trader_lock_sell(
        &mut self,
        max_sell_price: (GoodKind, f32, usize, f32),
    ) -> Result<String, ()> {
        println!("\n-----------------\nLock Sell...\n------------------");
        self.day();
        let token = self.markets[max_sell_price.2]
            .as_ref()
            .borrow_mut()
            .lock_sell(
                max_sell_price.0,
                max_sell_price.3,
                max_sell_price.1,
                String::from("Maccacaro"),
            );
        match token {
            Ok(token) => {
                self.last_daily_data.push(DailyData {
                    event: MarketEvent::LockSell,
                    amount_given: max_sell_price.3 as f64,
                    amount_received: max_sell_price.1 as f64,
                    kind_given: max_sell_price.0,
                    kind_received: GoodKind::EUR,
                });

                self.add_market_event();

                println!("Sell Locked");
                return Ok(token);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(());
            }
        }
    }

    pub fn trader_sell(
        &mut self,
        token: &String,
        max_sell_price: (GoodKind, f32, usize, f32),
    ) -> Result<(), ()> {
        let good = unitn_market_2022::good::good::Good::new(max_sell_price.0, max_sell_price.3);
        println!("-----------------\nSell...\n------------------");
        println!("Good: {:?}", good.clone());
        self.day();
        let res = self
            .markets
            .get(max_sell_price.2 as usize)
            .as_ref()
            .unwrap()
            .borrow_mut()
            .sell(token.clone(), &mut good.clone());
        match res {
            Ok(good_sell) => {
                self.last_daily_data.push(DailyData {
                    event: MarketEvent::Sell,
                    amount_given: max_sell_price.3 as f64,
                    amount_received: max_sell_price.1 as f64,
                    kind_given: max_sell_price.0,
                    kind_received: GoodKind::EUR,
                });

                self.add_market_event();

                self.set_good_quantity(
                    good.get_kind(),
                    self.get_good_quantity(good.get_kind()) - good.get_qty(),
                );
                self.set_good_quantity(
                    good_sell.get_kind(),
                    self.get_good_quantity(good_sell.get_kind()) + good_sell.get_qty(),
                );
                println!("Good Sell: {:?}, for {:?}", good.clone(), good_sell);
                return Ok(());
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(());
            }
        }
    }

    pub fn trader_wait_one_day(&mut self, markets: Vec<Rc<RefCell<dyn Market>>>) {
        println!("\n-----------------\nWaiting...\n------------------");

        self.last_daily_data.push(DailyData {
            event: MarketEvent::Wait,
            amount_given: 0.,
            amount_received: 0.,
            kind_given: GoodKind::EUR,
            kind_received: GoodKind::EUR,
        });

        self.day();
        wait_one_day!(markets[0].clone(), markets[1].clone());
        // wait_one_day!(markets[0].clone(), markets[1].clone(), markets[2].clone());
    }
}
