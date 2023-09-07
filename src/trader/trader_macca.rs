extern crate core;

use crate::data_models::market::{
    Currency, CurrencyData, DailyCurrencyData, DailyData, MarketData, MarketEvent,
};

use crate::data_models::market::TraderTrait;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Error;
use std::rc::Rc;
use unitn_market_2022;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::event::event::EventKind::{Bought, LockedBuy, LockedSell, Sold, Wait};
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{
    BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError,
};

use BVC::BVCMarket;
//use rcnz_market::rcnz::RCNZ;
use bose::market::BoseMarket;
use rand::{thread_rng, Rng};
use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    DEFAULT_GOOD_KIND,
};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::market::LockBuyError::BidTooLow;
use unitn_market_2022::{subscribe_each_other, wait_one_day};

// old/wrong crate
// use RCNZ::RCNZ;

const STARTING: f32 = 5000.0;
const MAX_DAYS: i32 = 1000;
const start_min: f32 = 100.0;
const start_max: f32 = 500.0;
const start_percentage: f32 = 2.0;

pub struct trade {
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

pub struct trader_maccacaro {
    markets: Vec<Rc<RefCell<dyn Market>>>,
    goods: HashMap<GoodKind, GoodMetadata>,
    traders_locked: HashMap<String, trade>,
    day: i32,
    name: String,
    last_daily_data: Vec<DailyData>,
    last_market_data: Vec<Vec<MarketData>>,
    min: f32,
    max: f32,
    percentage: f32,
    sell_locked: Vec<(GoodKind, f32, usize, f32, String)>,
    buy_locked: Vec<(GoodKind, f32, usize, f32, String)>,
    done: bool,
}

impl TraderTrait for trader_maccacaro {
    fn initialize_trader() -> Self {
        return trader_maccacaro::init_trader(trader_maccacaro::init_markets());
    }
    fn progess_day(&mut self, stratIndex: i32) {
        self.last_daily_data = Vec::new();
        self.last_market_data = Vec::new();
        self.Default();
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

impl trader_maccacaro {
    pub fn add_market_event(&mut self) {
        let mut data: Vec<MarketData> = Vec::new();

        self.markets.iter().enumerate().for_each(|(i, market)| {
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

        markets.push((BVCMarket::new_random()));
        markets.push((BoseMarket::new_random()));
        // markets.push((RCNZ::new_random()));

        // subscribe_each_other!(markets[0], markets[1], markets[2]);
        subscribe_each_other!(markets[0], markets[1]);
        //
        markets
    }

    pub fn init_trader(markets: Vec<Rc<RefCell<dyn Market>>>) -> trader_maccacaro {
        let mut trader = trader_maccacaro {
            markets: markets,
            goods: HashMap::new(),
            traders_locked: HashMap::new(),
            day: 0,
            name: String::from("Maccacaro"),
            last_daily_data: Vec::new(),
            last_market_data: Vec::new(),
            min: start_min,
            max: start_max,
            percentage: start_percentage,
            sell_locked: Vec::new(),
            buy_locked: Vec::new(),
            done: false,
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

    pub fn Default(&mut self) {
        if self.day < MAX_DAYS {
            let GoodKinds = [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
            let mut rng = thread_rng();
            let mut max_buy_price = (GoodKind::EUR, 0., 0, 0.);
            let mut max_sell_price = (GoodKind::EUR, 0., 0, 0.);

            let mut quantity1 = rng.gen_range(self.min..=self.max);

            for good in GoodKinds.iter() {
                for i in 0..self.markets.len() {
                    if *good != GoodKind::EUR {
                        let mut quantity_to_buy = quantity1 * good.get_default_exchange_rate();
                        let mut quantity_to_sell = quantity1 / good.get_default_exchange_rate();
                        let mut buy_price = 0.;
                        let mut sell_price = 0.;
                        let market = self.markets[i].clone();
                        buy_price = match market.borrow().get_buy_price(*good, quantity_to_buy) {
                            Ok(price) => price,
                            Err(e) => 0.,
                        };
                        sell_price = match market.borrow().get_sell_price(*good, quantity_to_sell) {
                            Ok(price) => price,
                            Err(e) => 0.,
                        };
                        if (buy_price / quantity_to_buy * 100.0 > max_buy_price.1
                            || max_buy_price.1 == 0.)
                            && buy_price > 0.
                        {
                            max_buy_price = (*good, buy_price, i, quantity_to_buy);
                        }
                        if (sell_price / quantity_to_sell * 100.0 > max_sell_price.1
                            || max_sell_price.1 == 0.)
                            && sell_price > 0.
                        {
                            max_sell_price = (*good, sell_price, i, quantity_to_sell);
                        }
                    }
                }
            }

            if self.sell_locked.is_empty() && self.buy_locked.is_empty() {
                let mut token;

                if max_buy_price.3 / max_buy_price.0.get_default_exchange_rate() < max_sell_price.1
                {
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
                        Err(e) => {}
                    }
                } else {
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
                        Err(e) => {}
                    }
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
                            Err(e) => {
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
                            Err(e) => {
                                self.sell_locked.pop();
                            }
                        }
                    } else {
                        self.sell_locked.pop();
                        println!("Not enough quantity")
                    }
                } else {
                    self.trader_wait_one_day(
                        self.markets[0].clone(),
                        self.markets[1].clone(),
                        self.markets[2].clone(),
                    );
                }
            }

            self.add_market_event();

            let mut upgrade = true;
            for good in GoodKinds.iter() {
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
                String::from("tt"),
            );
        match token {
            Ok(token) => {
                let trade = trade {
                    trans_kind: LockedBuy,
                    kind_giving: GoodKind::EUR,
                    kind_receiving: max_buy_price.0,
                    giving_quantity: max_buy_price.3,
                    receiving_quantity: max_buy_price.1,
                    market_index: max_buy_price.2,
                };

                self.last_daily_data.push(DailyData {
                    event: MarketEvent::LockBuy,
                    amount_given: max_buy_price.3 as f64,
                    amount_received: max_buy_price.1 as f64,
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
        let mut good = unitn_market_2022::good::good::Good::new(GoodKind::EUR, max_buy_price.1);
        println!("-----------------\nBuy...\n------------------");
        println!("Good: {:?}", good.clone());
        self.day();
        let res = self.markets[max_buy_price.2]
            .as_ref()
            .borrow_mut()
            .buy(token.clone(), &mut good.clone());
        match res {
            Ok(goodBought) => {
                let trade = trade {
                    trans_kind: Bought,
                    kind_giving: GoodKind::EUR,
                    kind_receiving: max_buy_price.0,
                    giving_quantity: max_buy_price.3,
                    receiving_quantity: max_buy_price.1,
                    market_index: max_buy_price.2,
                };

                self.last_daily_data.push(DailyData {
                    event: MarketEvent::Buy,
                    amount_given: max_buy_price.3 as f64,
                    amount_received: max_buy_price.1 as f64,
                    kind_given: GoodKind::EUR,
                    kind_received: max_buy_price.0,
                });

                self.add_market_event();

                self.set_good_quantity(
                    good.get_kind(),
                    self.get_good_quantity(good.get_kind()) - good.get_qty(),
                );
                self.set_good_quantity(
                    goodBought.get_kind(),
                    self.get_good_quantity(goodBought.get_kind()) + goodBought.get_qty(),
                );
                println!("Good Bought: {:?}, for {:?}", goodBought, good.clone());
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
                String::from("tt"),
            );
        match token {
            Ok(token) => {
                let trade = trade {
                    trans_kind: LockedSell,
                    kind_giving: max_sell_price.0,
                    kind_receiving: GoodKind::EUR,
                    giving_quantity: max_sell_price.3,
                    receiving_quantity: max_sell_price.1,
                    market_index: max_sell_price.2,
                };

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
        let mut good = unitn_market_2022::good::good::Good::new(max_sell_price.0, max_sell_price.3);
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
            Ok(goodSell) => {
                let trade = trade {
                    trans_kind: LockedSell,
                    kind_giving: max_sell_price.0,
                    kind_receiving: GoodKind::EUR,
                    giving_quantity: max_sell_price.3,
                    receiving_quantity: max_sell_price.1,
                    market_index: max_sell_price.2 as usize,
                };

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
                    goodSell.get_kind(),
                    self.get_good_quantity(goodSell.get_kind()) + goodSell.get_qty(),
                );
                println!("Good Sell: {:?}, for {:?}", good.clone(), goodSell);
                return Ok(());
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(());
            }
        }
    }

    pub fn trader_wait_one_day(
        &mut self,
        market_bose: Rc<RefCell<dyn Market>>,
        market_BVC: Rc<RefCell<dyn Market>>,
        market_RCNZ: Rc<RefCell<dyn Market>>,
    ) {
        println!("\n-----------------\nWaiting...\n------------------");

        self.last_daily_data.push(DailyData {
            event: MarketEvent::Wait,
            amount_given: 0.,
            amount_received: 0.,
            kind_given: GoodKind::EUR,
            kind_received: GoodKind::EUR,
        });

        self.day();
        wait_one_day!(market_bose, market_BVC, market_RCNZ);
    }
}
