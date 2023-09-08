use super::{get_default_exchange, TraderRicca};

use crate::trader::trader_ricca::consts::BVC_MAX_PERCENTAGE_BUY;
use std::cmp::Ordering;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good_kind::GoodKind;

impl TraderRicca {
    /*
        given a market, calculate the amount of a certain good that can be bought

        return a tuple with the quantity and the price
    */
    pub fn calculate_amount_to_buy(
        &self,
        market_index: usize,
        good_kind: GoodKind,
        quantity_to_sell: f32,
        max_wealth: f32,
    ) -> Option<(f32, f32)> {
        let good = self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .find(|g| g.good_kind == good_kind)
            .unwrap()
            .clone();
        let eur_owned = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();
        if good.exchange_rate_buy <= 0. || good.quantity <= 0. {
            return None;
        }
        let max_quantity_buyable = eur_owned * max_wealth / good.exchange_rate_buy;
        let mut max_quantity_market = good.quantity;
        if market_index == 0 {
            max_quantity_market *= BVC_MAX_PERCENTAGE_BUY;
        }

        //min between max_quantity and max_quantity_market
        let max_quantity = if max_quantity_buyable < max_quantity_market {
            max_quantity_buyable
        } else {
            max_quantity_market
        };

        /*
        println!(
            "quantity_to_sell:{}, Max_quantiry_market: {}, max_quantity_buyable: {} of {}",
            quantity_to_sell, max_quantity_market, max_quantity_buyable, good_kind
        );
        println!("exchange_rate_buy: {}", good.exchange_rate_buy);
         */

        if max_quantity < quantity_to_sell {
            let price = self.markets[market_index]
                .borrow()
                .get_buy_price(good_kind, max_quantity);
            match price {
                Ok(price) => {
                    if price > 0. {
                        return Some((max_quantity, price));
                    }
                    return None;
                }
                Err(_) => {
                    return None;
                }
            }
        } else {
            let price = self.markets[market_index]
                .borrow()
                .get_buy_price(good_kind, quantity_to_sell);

            match price {
                Ok(price) => {
                    return Some((quantity_to_sell, price));
                }
                Err(_) => {
                    return None;
                }
            }
        }
    }

    /*
       given a market, calculate the amount of a certain good that can be sold

       return a tuple with the quantity and the price
    */
    pub fn calculate_amount_to_sell(
        &self,
        market_index: usize,
        good_kind: GoodKind,
    ) -> Option<(f32, f32)> {
        let goods = self.markets[market_index].borrow().get_goods();
        //iter goods and find the good_kind

        let mut quantity_of_eur_of_market = 0.;
        for good in goods {
            if good.good_kind == DEFAULT_GOOD_KIND {
                quantity_of_eur_of_market = good.quantity;
            }
        }

        let goods = self.markets[market_index].borrow().get_goods();

        for good in goods {
            if good.good_kind == good_kind {
                let mut max_quantity = quantity_of_eur_of_market / good.exchange_rate_sell;
                if market_index == 1 {
                    max_quantity = quantity_of_eur_of_market * good.exchange_rate_sell;
                } else if market_index == 0 {
                    max_quantity = max_quantity * BVC_MAX_PERCENTAGE_BUY;
                }

                let quantity = max_quantity;
                if quantity <= 0. {
                    return None;
                }

                let sell_price = self.markets[market_index]
                    .borrow()
                    .get_sell_price(good_kind, quantity);

                match sell_price {
                    Ok(price) => {
                        if price > 0. {
                            return Some((quantity, price));
                        }
                        return None;
                    }
                    Err(_) => {
                        return None;
                    }
                }
            }
        }

        return None;
    }

    /*
        find the market with the lowest buy price of a certain good

        return a tuple with the price and the index of the market
    */
    pub fn find_buy_cheapest_quantity(
        &mut self,
        good_kind: &GoodKind,
        excluded_markets: Vec<usize>,
    ) -> (f32, usize) {
        let values = self
            .markets
            .iter()
            .enumerate()
            .filter(|(i, _)| !excluded_markets.contains(i))
            .map(|(i, market)| (market.borrow().get_buy_price(*good_kind, 1.), i))
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

    /*
       find the market with the highest sell price of a certain good

       return a tuple with the price and the index of the market
    */
    pub fn find_sell_highest_quantity(
        &mut self,
        good_kind: &GoodKind,
        excluded_markets: Vec<usize>,
    ) -> (f32, usize) {
        if excluded_markets.len() + 1 == self.markets.len() {
            let index_not_excluded = self
                .markets
                .iter()
                .enumerate()
                .find(|(i, _)| !excluded_markets.contains(i))
                .unwrap()
                .0;

            let price = self.markets[index_not_excluded]
                .borrow()
                .get_sell_price(*good_kind, 1.)
                .unwrap();
            return (price, index_not_excluded);
        }

        self.markets
            .iter()
            .enumerate()
            .filter(|(i, _)| !excluded_markets.contains(i))
            .map(|(i, market)| (market.borrow().get_sell_price(*good_kind, 1.).unwrap(), i))
            .max_by(|(price1, _), (price2, _)| {
                if *price1 <= 0. {
                    return Ordering::Less;
                }
                price1.partial_cmp(&price2).unwrap()
            })
            .unwrap()
    }

    /*
        given a market, calculate the value of all the goods converted with the standard exchange rate

        return the value
    */
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

    /*
       given a market, calculate the value of the kind that is worth the least EUR converted with the standard exchange rate

       return a tuple with the value and the kind
    */
    pub fn calculate_market_lowest_kind(&self, market_index: usize) -> (f32, GoodKind) {
        let mut wealth_of_kind = vec![];
        self.markets[market_index]
            .borrow()
            .get_goods()
            .iter()
            .for_each(|g| {
                if g.good_kind != GoodKind::EUR {
                    wealth_of_kind
                        .push((g.quantity * get_default_exchange(g.good_kind), g.good_kind));
                }
            });

        wealth_of_kind.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        wealth_of_kind[0]
    }

    /*
       given a market, calculate the value of the kind that is worth the most EUR converted with the standard exchange rate

       return a tuple with the value and the kind
    */
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

    pub fn other_than_eur(&self) -> bool {
        self.goods
            .iter()
            .any(|(k, v)| *k != GoodKind::EUR && v.good.get_qty() > 0.)
    }

    /*
        find the biggest difference in sell and buy price of a certain good on different markets

        return a tuple with the index of the market with the highest sell price, the index of the market with the lowest buy price, the sell price and the buy price
    */
    pub fn find_biggest_difference(
        &mut self,
        good_kind: &GoodKind,
        exclude_sell: Vec<usize>,
        exclude_buy: Vec<usize>,
    ) -> Option<(usize, usize, f32, f32)> {
        if (exclude_sell.len() == self.markets.len()) || (exclude_buy.len() == self.markets.len()) {
            return None;
        }
        let max_sell = self.find_sell_highest_quantity(good_kind, exclude_sell);
        let min_buy = self.find_buy_cheapest_quantity(good_kind, exclude_buy.clone());

        if max_sell.0 <= 0. || min_buy.0 <= 0. {
            return None;
        }
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

    /*
        find the best goodkind to buy, based on the function self.find_biggest_difference()

        return a tuple with the goodkind and the tuple returned by find_biggest_difference
    */
    pub fn find_best_goodkind(
        &mut self,
        excluded_markets: Vec<usize>,
    ) -> Option<(GoodKind, (usize, usize, f32, f32))> {
        let mut rates: Vec<(GoodKind, (usize, usize, f32, f32))> = Vec::new();

        let good_kinds = [GoodKind::USD, GoodKind::YUAN, GoodKind::YEN];

        for good_kind in good_kinds {
            if good_kind != GoodKind::EUR {
                let rate = self.find_biggest_difference(
                    &good_kind,
                    excluded_markets.clone(),
                    excluded_markets.clone(),
                );
                /*
                println!("rate: {:?}", rate);
                println!("goodKind: {:?}", goodKind);
                println!("target_quantity: {:?}", target_quantity.clone());
                 */
                if rate.is_some() {
                    rates.push((good_kind.clone(), rate.unwrap()));
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

        return rates.pop();
    }

    /*
        given a market, calculate witch kind is worth the least EUR converted with the standard exchange rate expressed as a percentage of the total wealth of the market

        return a tuple with the percentage and the kind
    */
    pub fn calculate_market_lowest_kind_percentage(&self, market_index: usize) -> (f32, GoodKind) {
        let wealth_of_market = self.calculate_market_wealth(market_index);
        let lowest_good = self.calculate_market_lowest_kind(market_index);

        return (wealth_of_market / lowest_good.0, lowest_good.1);
    }
}
