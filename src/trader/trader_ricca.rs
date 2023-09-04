/*
Obbiettivo: creare un trader che parta con x euro e ne guadagni il più possibile in un tempo limitato (y giorni)
 */

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
use RCNZ::RCNZ;

const STARTING_EUR: f32 = 500000.0;
const MAX_DAYS: i32 = 1000;

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

pub struct trader_struct {
    markets: Vec<Rc<RefCell<dyn Market>>>,
    goods: HashMap<GoodKind, GoodMetadata>,
    traders_locked: HashMap<String, trade>,
    day: i32,
    name: String,
    consecutive_wait: i32,
    not_optimal_counter: i32,
    last_daily_data: DailyData,
}

impl TraderTrait for trader_struct {
    fn initialize_trader(stratIndex: i32) -> Self {
        return trader_struct::init_trader(trader_struct::init_markets());
    }
    fn progess_day(&mut self) {
        self.main_strat_day();
    }

    fn get_daily_data(&self) -> DailyData {
        self.last_daily_data.clone()
    }

    fn get_market_data(&self) -> Vec<MarketData> {
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

        return data;
    }

    fn get_trader_data(&self) -> CurrencyData {
        let mut currency_data = CurrencyData {
            eur: self.goods.get(&GoodKind::EUR).unwrap().good.get_qty() as f64,
            usd: self.goods.get(&GoodKind::USD).unwrap().good.get_qty() as f64,
            yen: self.goods.get(&GoodKind::YEN).unwrap().good.get_qty() as f64,
            yuan: self.goods.get(&GoodKind::YUAN).unwrap().good.get_qty() as f64,
        };

        return currency_data;
    }
}

pub fn get_default_exchange(goodKind: GoodKind) -> f32 {
    match goodKind {
        GoodKind::EUR => 1.,
        GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
        GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
        GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    }
}

impl trader_struct {
    pub fn init_markets() -> Vec<Rc<RefCell<dyn Market>>> {
        let mut markets: Vec<Rc<RefCell<dyn Market>>> = Vec::new();

        markets.push((BVCMarket::new_random()));
        markets.push((BoseMarket::new_random()));
        markets.push((RCNZ::new_random()));

        subscribe_each_other!(markets[0], markets[1], markets[2]);
        //subscribe_each_other!(markets[0], markets[1]);

        markets
    }

    pub fn init_trader(markets: Vec<Rc<RefCell<dyn Market>>>) -> trader_struct {
        let mut trader = trader_struct {
            markets: markets,
            goods: HashMap::new(),
            traders_locked: HashMap::new(),
            day: 0,
            name: String::from("Emanuele"),
            consecutive_wait: 0,
            not_optimal_counter: 0,
            last_daily_data: DailyData {
                event: MarketEvent::Wait,
                amount_given: 0.,
                amount_received: 0.,
                kind_given: GoodKind::EUR,
                kind_received: GoodKind::EUR,
            },
        };

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

    pub fn main_strat_day(&mut self) {
        if (self.day < MAX_DAYS) {
            let starting_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            self.buy_low_sell_high_strat();

            let ending_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            if !(starting_eur < ending_eur) {
                /*
                if (self.other_than_EUR()) {
                    self.sell_all_strat();
                } else if self.consecutive_wait < 10 {
                    self.consecutive_wait += 1;

                    //wait_one_day!(self.markets[0],self.markets[1], self.markets[2]);
                    wait_one_day!(self.markets[0],self.markets[1]);

                    self.day_passed();
                } else {
                    //random f32 between 0 and 1
                    let mut rng = rand::thread_rng();
                    let random_f32: f32 = rng.gen();
                    if random_f32 < 0.5 {
                        self.buy_all_of_kind_strat(0.2);
                        self.buy_all_of_kind_strat(0.2);
                    } else {
                        self.random_buy_strat();
                    }



                }

                 */

                if self.not_optimal_counter > 5 {
                    self.sell_all_strat();
                    self.not_optimal_counter = 0;
                }

                let mut rng = rand::thread_rng();
                let random_f32: f32 = rng.gen();
                if random_f32 < 0.4 {
                    println!("--------------------------------------------\nBUY ALL OF KIND\n--------------------------------------------");

                    self.buy_all_of_kind_strat(0.2);
                    self.buy_all_of_kind_strat(0.2);
                } else if random_f32 < 0.8 {
                    println!("--------------------------------------------\nRANDOM BUY\n--------------------------------------------");

                    self.random_buy_strat();
                } else {
                    println!("--------------------------------------------\nWAIT\n--------------------------------------------");
                    wait_one_day!(self.markets[0], self.markets[1]);
                }

                //print all goods
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");

                self.not_optimal_counter += 1;
            } else {
                //print all goods
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");
                self.not_optimal_counter = 0;
            }
        } else if (self.day == MAX_DAYS) {
            self.sell_all_strat();
        } else {
            //trader done
        }
    }

    pub fn buy_all_of_kind_strat(&mut self, percentage_of_wealth: f32) {
        /*
        let mut vec_of_goods_to_buy = vec![GoodKind::USD, GoodKind::YUAN, GoodKind::YEN];
        //pick random item from vec
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..vec_of_goods_to_buy.len());
        let good_to_buy = vec_of_goods_to_buy.remove(random_index);

         */

        let mut vec_of_lowest_goods_of_markets_percentage = vec![];
        for i in 0..self.markets.len() {
            vec_of_lowest_goods_of_markets_percentage
                .push((self.calculate_market_lowest_kind_percentage(i), i));
        }

        let mut vec_of_lowest_goods_of_markets = vec![];
        for i in 0..self.markets.len() {
            vec_of_lowest_goods_of_markets.push((self.calculate_market_lowest_kind(i), i));
        }

        vec_of_lowest_goods_of_markets_percentage
            .sort_by(|((a, _), _), ((b, _), _)| a.partial_cmp(b).unwrap());
        let lowest_kind_market = vec_of_lowest_goods_of_markets_percentage[0];

        let market_index_lowest = lowest_kind_market.1;
        let lowest_kind = lowest_kind_market.0 .1;

        let buy_price = self.markets[market_index_lowest]
            .borrow()
            .get_buy_price(lowest_kind, 1.)
            .unwrap();
        let sell_price = self.find_sell_highest_quantity(&lowest_kind, vec![market_index_lowest]);

        if buy_price < sell_price.0 * 1.1 {
            let quantity = self
                .calculate_amount_to_buy(
                    market_index_lowest,
                    lowest_kind,
                    f32::INFINITY,
                    percentage_of_wealth,
                )
                .0;
            let price_to_pay = self.markets[market_index_lowest]
                .borrow()
                .get_buy_price(lowest_kind, quantity)
                .unwrap();
            let token_buy = self.lock_buy(market_index_lowest, lowest_kind, quantity, price_to_pay);
            match token_buy {
                Ok(_) => {
                    self.buy(token_buy.unwrap());
                }
                Err(_) => {
                    println!(
                        "BUYING {:?} FROM MARKET {} FOR {} EUR FAILED",
                        lowest_kind,
                        market_index_lowest,
                        quantity * buy_price
                    );
                    panic!(
                        "BUYING {:?} FROM MARKET {} FOR {} EUR FAILED",
                        lowest_kind,
                        market_index_lowest,
                        quantity * buy_price
                    );
                }
            }
        } else {
            println!(
                "not good enough price to buy {:?} from market {}, high sell: {:?}, buy :{}",
                lowest_kind, market_index_lowest, sell_price, buy_price
            );
            self.markets.iter().enumerate().for_each(|(i, market)| {
                let market = market.borrow();
                println!("Market {}: {:?}", market.get_name(), market.get_goods());
            });
        }
    }

    pub fn calculate_market_lowest_kind_percentage(&self, market_index: usize) -> (f32, GoodKind) {
        let wealth_of_market = self.calculate_market_wealth(market_index);
        let lowest_good = self.calculate_market_lowest_kind(market_index);

        return (wealth_of_market / lowest_good.0, lowest_good.1);
    }

    pub fn sell_all_strat(&mut self) {
        let mut vec_of_goods_to_sell = vec![];
        self.goods.iter().for_each(|(k, v)| {
            if *k != GoodKind::EUR && v.good.get_qty() > 0. {
                vec_of_goods_to_sell.push(*k);
                println!("TRYING FORCED SELL ON: {:?}", k);
            }
        });

        vec_of_goods_to_sell.iter().for_each(|k| {
            self.sell_to_highest(*k, vec![], 1.);
        });
    }

    pub fn random_buy_strat(&mut self) {
        let goods = vec![GoodKind::USD, GoodKind::YUAN, GoodKind::YEN];
        let quantity_of_EUR = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();
        //randomly buy all kinds of goods
        goods.iter().for_each(|(k)| {
            let y: f32 = rand::thread_rng().gen();
            let v = (y * 0.10) + 0.05;

            let quantity_of_EUR_to_spend = quantity_of_EUR * v;
            self.buy_random(*k, quantity_of_EUR_to_spend, vec![]);
        });
    }

    pub fn buy_low_sell_high_strat(&mut self) {
        let rate = self.find_best_goodKind();
        if (rate.1 .2 > rate.1 .3) {
            let mut quantity_gain_to_sell = self.calculate_amount_to_sell(rate.1 .0, rate.0);
            let quantity_price =
                self.calculate_amount_to_buy(rate.1 .1, rate.0, quantity_gain_to_sell.0, 0.8);
            //println!("quantity_price: {:?}", quantity_price);

            if quantity_price.0 != quantity_gain_to_sell.0 {
                quantity_gain_to_sell = (
                    quantity_price.0,
                    self.markets[rate.1 .0]
                        .borrow()
                        .get_sell_price(rate.0, quantity_price.0)
                        .unwrap(),
                );
            }

            let gain = quantity_gain_to_sell.1;
            let price = quantity_price.1;
            let quantity = quantity_gain_to_sell.0;

            if quantity > 0.01 && gain > price * 1.02 {
                let token_sell = self.lock_sell(rate.1 .0, rate.0, quantity, gain);
                let mut token_buy = self.lock_buy(rate.1 .1, rate.0, quantity, price);
                match token_buy {
                    Err(BidTooLow {
                        lowest_acceptable_bid: lowest,
                        ..
                    }) => {
                        println!("Bid too low, lowest acceptable bid is: {}", lowest);
                        if gain > lowest {
                            token_buy = self.lock_buy(rate.1 .1, rate.0, quantity, lowest);
                            self.buy(token_buy.unwrap());
                            self.sell(token_sell.unwrap());
                        } else {
                            panic!("Price changed wrongly");
                        }
                    }
                    Err(_) => {
                        //panic
                        panic!("Unexpected error");
                    }
                    _ => {
                        self.buy(token_buy.unwrap());
                        self.sell(token_sell.unwrap());
                    }
                }

                /*

                //print all goods
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });

                 */
            }

            //break;
        }
    }

    pub fn buy_random(
        &mut self,
        good_kind: GoodKind,
        quantity_of_EUR_to_spend: f32,
        mut exclude_markets: Vec<usize>,
    ) {
        if exclude_markets.len() == self.markets.len() {
            println!("No market to buy from");
            return;
        }

        let best_deal = self.find_buy_cheapest_quantity(&good_kind, exclude_markets.clone());
        let quantity = quantity_of_EUR_to_spend / best_deal.0;
        let mut price = self.markets[best_deal.1]
            .borrow()
            .get_buy_price(good_kind, quantity);
        match price {
            Err(_) => {
                exclude_markets.push(best_deal.1);
                self.buy_random(good_kind, quantity_of_EUR_to_spend, exclude_markets);
            }
            _ => {
                let price = price.unwrap();
                if price * 1.05 > quantity_of_EUR_to_spend {
                    let token_buy = self.lock_buy(best_deal.1, good_kind, quantity, price);
                    match token_buy {
                        Ok(_) => {
                            self.buy(token_buy.unwrap());
                        }
                        Err(_) => {
                            exclude_markets.push(best_deal.1);
                            self.buy_random(good_kind, quantity_of_EUR_to_spend, exclude_markets);
                        }
                    }
                }
            }
        }
    }

    pub fn sell_to_highest(
        &mut self,
        good_kind: GoodKind,
        mut excluded_markets: Vec<usize>,
        percentage: f32,
    ) -> bool {
        if excluded_markets.len() == self.markets.len() {
            return false;
        }

        let quantity = self.goods.get(&good_kind).unwrap().good.get_qty() * percentage;
        let best_deal = self.find_sell_highest_quantity(&good_kind, excluded_markets.clone());
        let price = self.markets[best_deal.1]
            .borrow()
            .get_sell_price(good_kind, quantity);
        match price {
            Err(_) => {
                excluded_markets.push(best_deal.1);
                self.sell_to_highest(good_kind, excluded_markets, percentage);
            }
            _ => {
                println!(
                    "PRice: {:?}, quantity: {}",
                    price.clone().unwrap(),
                    quantity
                );
                let token_sell = self.lock_sell(best_deal.1, good_kind, quantity, price.unwrap());
                match token_sell {
                    Err(_) => {
                        excluded_markets.push(best_deal.1);
                        self.sell_to_highest(good_kind, excluded_markets, percentage);
                    }
                    _ => {
                        self.sell(token_sell.unwrap());
                    }
                }
            }
        }

        return true;
    }

    pub fn calculate_market_wealth(&self, market_index: usize) -> f32 {
        let mut wealth = 0.;
        self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .for_each(|g| {
                wealth += g.quantity * get_default_exchange(g.good_kind);
            });

        wealth
    }

    pub fn calculate_market_lowest_kind(&self, market_index: usize) -> (f32, GoodKind) {
        let mut wealth_of_kind = vec![];
        self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .for_each(|g| {
                wealth_of_kind.push((g.quantity * get_default_exchange(g.good_kind), g.good_kind));
            });

        wealth_of_kind.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        wealth_of_kind[0]
    }

    pub fn calculate_market_higher_kind(&self, market_index: usize) -> (f32, GoodKind) {
        let mut wealth_of_kind = vec![];
        self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .for_each(|g| {
                wealth_of_kind.push((g.quantity * get_default_exchange(g.good_kind), g.good_kind));
            });

        wealth_of_kind.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        wealth_of_kind[wealth_of_kind.len() - 1]
    }

    pub fn other_than_EUR(&self) -> bool {
        self.goods
            .iter()
            .any(|(k, v)| *k != GoodKind::EUR && v.good.get_qty() > 0.)
    }

    pub fn day_passed(&mut self) {
        self.day += 1;
    }

    pub fn calculate_amount_to_buy(
        &self,
        market_index: usize,
        good_kind: GoodKind,
        quantity_to_sell: f32,
        max_wealth: f32,
    ) -> (f32, f32) {
        let good = self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .find(|g| g.good_kind == good_kind)
            .unwrap()
            .clone();
        let eur_owned = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();
        let max_quantity_buyable = (eur_owned * max_wealth / good.exchange_rate_buy);
        let mut max_quantity_market = good.quantity;
        if market_index == 0 {
            max_quantity_market *= 0.1;
        }

        //min between max_quantity and max_quantity_market
        let max_quantity = if max_quantity_buyable < max_quantity_market {
            max_quantity_buyable
        } else {
            max_quantity_market
        };

        println!(
            "quantity_to_sell:{}, Max_quantiry_market: {}, max_quantity_buyable: {} of {}",
            quantity_to_sell, max_quantity_market, max_quantity_buyable, good_kind
        );
        println!("exchange_rate_buy: {}", good.exchange_rate_buy);

        if max_quantity < quantity_to_sell {
            return (
                max_quantity,
                self.markets[market_index]
                    .borrow()
                    .get_buy_price(good_kind, max_quantity)
                    .unwrap(),
            );
        } else {
            return (
                quantity_to_sell,
                self.markets[market_index]
                    .borrow()
                    .get_buy_price(good_kind, quantity_to_sell)
                    .unwrap(),
            );
        }
    }

    pub fn calculate_amount_to_sell(&self, market_index: usize, good_kind: GoodKind) -> (f32, f32) {
        let mut goods = self.markets[market_index].borrow().get_goods();
        //iter goods and find the good_kind

        let quantity_of_EUR = self.goods.get(&DEFAULT_GOOD_KIND).unwrap().good.get_qty();
        let mut quantity_of_EUR_of_market = 0.;
        for good in goods {
            if good.good_kind == DEFAULT_GOOD_KIND {
                quantity_of_EUR_of_market = good.quantity;
            }
        }

        let mut goods = self.markets[market_index].borrow().get_goods();
        for good in goods {
            if good.good_kind == good_kind {
                let mut max_quantity = (quantity_of_EUR_of_market / good.exchange_rate_sell);
                if market_index == 1 {
                    max_quantity = (quantity_of_EUR_of_market * good.exchange_rate_sell)
                } else if market_index == 0 {
                    max_quantity = max_quantity * 0.25;
                }

                let quantity = max_quantity;

                let mut sell_price = self.markets[market_index]
                    .borrow()
                    .get_sell_price(good_kind, quantity)
                    .unwrap();

                return (quantity, sell_price);
            }
        }

        return (-1., 0.);
    }

    pub fn find_buy_cheapest_quantity(
        &mut self,
        goodKind: &GoodKind,
        excluded_markets: Vec<usize>,
    ) -> (f32, usize) {
        /*
        self.markets.iter().enumerate().filter(|(i,_)| !excluded_markets.contains(i)).map(|(i, market)| {
            (market.borrow().get_buy_price(*goodKind, 1.).unwrap(), i)
        }).min_by(|(price1, _), (price2, _)| {
            price1.partial_cmp(&price2).unwrap()
        }).unwrap()

         */
        let values = self
            .markets
            .iter()
            .enumerate()
            .filter(|(i, _)| !excluded_markets.contains(i))
            .map(|(i, market)| (market.borrow().get_buy_price(*goodKind, 1.), i))
            .min_by(|(price1, _), (price2, _)| {
                if price1.is_err() {
                    return Ordering::Greater;
                }

                if price2.is_err() {
                    return Ordering::Less;
                }
                price1
                    .clone()
                    .unwrap()
                    .partial_cmp(&price2.clone().unwrap())
                    .unwrap()
            })
            .unwrap();

        if values.0.is_err() {
            return (-1., 0);
        } else {
            return (values.0.unwrap(), values.1);
        }
    }

    pub fn find_sell_highest_quantity(
        &mut self,
        goodKind: &GoodKind,
        excluded_markets: Vec<usize>,
    ) -> (f32, usize) {
        self.markets
            .iter()
            .enumerate()
            .filter(|(i, _)| !excluded_markets.contains(i))
            .map(|(i, market)| (market.borrow().get_sell_price(*goodKind, 1.).unwrap(), i))
            .max_by(|(price1, _), (price2, _)| price1.partial_cmp(&price2).unwrap())
            .unwrap()
    }

    //find the biggest difference in sell and buy price of a certain good on different markets
    pub fn find_biggest_difference(
        &mut self,
        goodKind: &GoodKind,
        exclude_sell: Vec<usize>,
        mut exclude_buy: Vec<usize>,
    ) -> Option<(usize, usize, f32, f32)> {
        let mut max_sell = self.find_sell_highest_quantity(goodKind, exclude_sell);
        let mut min_buy = self.find_buy_cheapest_quantity(goodKind, exclude_buy.clone());

        /*
        if (max_sell.1 == min_buy.1){
            exclude_buy.push(max_sell.1);
            min_buy = self.find_buy_cheapest_quantity(goodKind,  exclude_buy);

        }

         */
        let tuple = (max_sell.1, min_buy.1, max_sell.0, min_buy.0);
        return Some(tuple);
    }

    pub fn get_total_wealth(&mut self) -> f32 {
        let mut total = 0.0;
        for (_, good) in self.goods.iter() {
            total += good.good.get_qty() * good.exchange_rate;
        }
        total
    }

    pub fn find_best_goodKind(&mut self) -> (GoodKind, (usize, usize, f32, f32)) {
        let mut rates: Vec<(GoodKind, (usize, usize, f32, f32))> = Vec::new();

        let goodKinds = [GoodKind::USD, GoodKind::YUAN, GoodKind::YEN];

        for goodKind in goodKinds {
            if goodKind != GoodKind::EUR {
                let rate = self.find_biggest_difference(&goodKind, vec![], vec![]);
                /*
                println!("rate: {:?}", rate);
                println!("goodKind: {:?}", goodKind);
                println!("target_quantity: {:?}", target_quantity.clone());
                 */
                if rate.is_some() {
                    rates.push((goodKind.clone(), rate.unwrap()));
                }
            }
        }

        rates.sort_by(
            |(_, (_, _, rate1_sell, rate1_buy)), (_, (_, _, rate2_sell, rate2_buy))| {
                (rate1_sell - rate1_buy)
                    .partial_cmp(&(rate2_sell - rate2_buy))
                    .unwrap()
            },
        );
        //print the vector rates

        /*
        rates.iter().for_each(|(goodKind,(_,_,rate_sell,rate_buy))| {
            println!("goodKind: {:?}", goodKind);
            println!("rate_sell: {:?}", rate_sell);
            println!("rate_buy: {:?}", rate_buy);
        });

         */

        return rates.pop().unwrap();
    }

    pub fn lock_buy(
        &mut self,
        market_index: usize,
        goodKind: GoodKind,
        quantity_to_buy: f32,
        price_to_pay: f32,
    ) -> Result<String, LockBuyError> {
        let event = self.markets[market_index].borrow_mut().lock_buy(
            goodKind,
            quantity_to_buy,
            price_to_pay,
            self.name.clone(),
        );

        match event {
            Ok(token) => {
                let trade = trade {
                    trans_kind: LockedBuy,
                    kind_giving: GoodKind::EUR,
                    kind_receiving: goodKind,
                    giving_quantity: price_to_pay,
                    receiving_quantity: quantity_to_buy,
                    market_index: market_index,
                };
                self.traders_locked.insert(token.clone(), trade);

                self.last_daily_data = DailyData {
                    event: MarketEvent::LockBuy,
                    amount_given: price_to_pay as f64,
                    amount_received: quantity_to_buy as f64,
                    kind_given: GoodKind::EUR,
                    kind_received: goodKind,
                };

                self.day_passed();
                println!(
                    "{} locked buy {} {} for {} EUR on {}",
                    self.name, quantity_to_buy, goodKind, price_to_pay, market_index
                );
                println!("DAY NUMBER: {}", self.day);
                return Ok(token);
            }
            Err(e) => {
                println!("on market {} Error: {:?}", market_index, e);
                return Err(e);
            }
        }
    }

    pub fn buy(&mut self, token: String) -> Result<(), BuyError> {
        let trade = self.traders_locked.get_mut(&*token).unwrap();
        if (trade.giving_quantity > self.goods.get(&trade.kind_giving).unwrap().good.get_qty()) {
            println!("Not enough money");
            return Err(BuyError::InsufficientGoodQuantity {
                contained_quantity: 0.0,
                pre_agreed_quantity: -1.0,
            });
        } else {
            let res = self.markets[trade.market_index].borrow_mut().buy(
                token.clone(),
                &mut self
                    .goods
                    .get_mut(&GoodKind::EUR)
                    .unwrap()
                    .good
                    .split(trade.giving_quantity)
                    .unwrap(),
            );
            match res {
                Ok(good) => {
                    self.goods
                        .get_mut(&trade.kind_receiving)
                        .unwrap()
                        .good
                        .merge(good);

                    self.last_daily_data = DailyData {
                        event: MarketEvent::Buy,
                        amount_given: trade.giving_quantity as f64,
                        amount_received: trade.receiving_quantity as f64,
                        kind_given: GoodKind::EUR,
                        kind_received: trade.kind_receiving,
                    };

                    println!(
                        "{} bought {} {} for {} EUR on {}",
                        self.name,
                        trade.receiving_quantity,
                        trade.kind_receiving,
                        trade.giving_quantity,
                        trade.market_index
                    );
                    self.traders_locked.remove(&*token);
                    self.day_passed();
                    println!("DAY NUMBER: {}", self.day);

                    return Ok(());
                }
                Err(e) => {
                    match e {
                        BuyError::ExpiredToken { .. } => {
                            println!("Expired token");
                            self.traders_locked.remove(&*token);
                        }
                        _ => {
                            println!("Error: {:?}", e);
                        }
                    }
                    return Err(e);
                }
            }
        }
    }

    pub fn lock_sell(
        &mut self,
        market_index: usize,
        goodKind: GoodKind,
        quantity_to_sell: f32,
        price_wanted: f32,
    ) -> Result<String, LockSellError> {
        let event = self.markets[market_index].borrow_mut().lock_sell(
            goodKind,
            quantity_to_sell,
            price_wanted,
            self.name.clone(),
        );
        match event {
            Ok(token) => {
                let trade = trade {
                    trans_kind: LockedSell,
                    kind_giving: goodKind,
                    kind_receiving: GoodKind::EUR,
                    giving_quantity: quantity_to_sell,
                    receiving_quantity: price_wanted,
                    market_index: market_index,
                };

                self.last_daily_data = DailyData {
                    event: MarketEvent::LockSell,
                    amount_given: quantity_to_sell as f64,
                    amount_received: price_wanted as f64,
                    kind_given: goodKind,
                    kind_received: GoodKind::EUR,
                };

                self.traders_locked.insert(token.clone(), trade);
                self.day_passed();
                println!(
                    "{} locked sell {} {} for {} EUR on {}",
                    self.name, quantity_to_sell, goodKind, price_wanted, market_index
                );
                println!("DAY NUMBER: {}", self.day);
                return Ok(token);
            }
            Err(e) => {
                println!("on market {} Error: {:?}", market_index, e);
                return Err(e);
            }
        }
    }

    pub fn sell(&mut self, token: String) -> Result<(), SellError> {
        let trade = self.traders_locked.get_mut(&*token).unwrap();
        if (trade.giving_quantity > self.goods.get(&trade.kind_giving).unwrap().good.get_qty()) {
            println!("Not enough money");
            return Err(SellError::InsufficientGoodQuantity {
                contained_quantity: 0.0,
                pre_agreed_quantity: -1.0,
            });
        } else {
            let mut selling_good = self
                .goods
                .get_mut(&trade.kind_giving)
                .unwrap()
                .good
                .split(trade.giving_quantity)
                .unwrap();
            let res = self.markets[trade.market_index]
                .borrow_mut()
                .sell(token.clone(), &mut selling_good.clone());
            match res {
                Ok(good) => {
                    self.goods
                        .get_mut(&trade.kind_receiving)
                        .unwrap()
                        .good
                        .merge(good);

                    self.last_daily_data = DailyData {
                        event: MarketEvent::Sell,
                        amount_given: trade.giving_quantity as f64,
                        amount_received: trade.receiving_quantity as f64,
                        kind_given: trade.kind_giving,
                        kind_received: GoodKind::EUR,
                    };

                    println!(
                        "{} sold {} {} for {} EUR on {}",
                        self.name,
                        trade.giving_quantity,
                        trade.kind_giving,
                        trade.receiving_quantity,
                        trade.market_index
                    );
                    self.traders_locked.remove(&*token);
                    self.day_passed();
                    println!("DAY NUMBER: {}", self.day);
                    return Ok(());
                }
                Err(e) => {
                    match e {
                        SellError::ExpiredToken { .. } => {
                            println!("Expired token");

                            self.goods
                                .get_mut(&trade.kind_giving)
                                .unwrap()
                                .good
                                .merge(selling_good);
                            self.traders_locked.remove(&*token);
                        }
                        _ => {
                            println!("Error: {:?}", e);
                            self.goods
                                .get_mut(&trade.kind_giving)
                                .unwrap()
                                .good
                                .merge(selling_good);
                        }
                    }

                    return Err(e);
                }
            }
        }
    }
}
