use crate::trader::trader_ricca::consts::{
    LOW_BUY_HIGH_SELL_MIN_GAIN, LOW_BUY_HIGH_SELL_MIN_QUANTITY, MAIN_ONE_KIND_PICK_PERCENTAGE,
    MAIN_ONE_KIND_WEALTH_PERCENTAGE, MAIN_RANDOM_BUY_PICK_PERCENTAGE, MAX_DAYS,
    NOT_OPTIMAL_TREEHOLD, ONE_KIND_PICK_PERCENTAGE, ONE_KIND_RESELLABLE_TREEHOLD,
    ONE_KIND_WEALTH_PERCENTAGE, RANDOM_BUY_MIN_MAX_WEALTH_PERCENTAGE, RANDOM_BUY_PICK_PERCENTAGE,
    RANDOM_BUY_RESELLABLE_TREEHOLD,
};

use crate::trader::trader_ricca::trader_main::TraderRicca;
use rand::Rng;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::LockBuyError::BidTooLow;

impl TraderRicca {
    pub fn end_game_sell_all(&mut self) {
        while !self.done {
            self.sell_all_strat();
            if !(self.other_than_eur()) {
                self.done = true;
            } else {
                self.wait_update();
            }
        }
    }

    pub fn buy_all_of_kind_strat(
        &mut self,
        percentage_of_wealth: f32,
        excluded_markets: Vec<usize>,
    ) -> bool {
        if excluded_markets.len() == self.markets.len() - 1 {
            return false;
        }

        let mut vec_of_lowest_goods_of_markets_percentage = vec![];
        for i in 0..self.markets.len() {
            //if i is not inside excluded_markets
            if !excluded_markets.contains(&i) {
                vec_of_lowest_goods_of_markets_percentage
                    .push((self.calculate_market_lowest_kind_percentage(i), i));
            }
        }

        vec_of_lowest_goods_of_markets_percentage
            .sort_by(|((a, _), _), ((b, _), _)| a.partial_cmp(b).unwrap());
        let lowest_kind_market = vec_of_lowest_goods_of_markets_percentage[0];

        let market_index_lowest = lowest_kind_market.1;
        let lowest_kind = lowest_kind_market.0 .1;

        let buy_price = self.markets[market_index_lowest]
            .borrow()
            .get_buy_price(lowest_kind, 1.);

        let buy_price = match buy_price {
            Ok(price) => price,
            Err(_) => f32::INFINITY,
        };

        let mut sell_exclude = excluded_markets.clone();
        sell_exclude.push(market_index_lowest);

        let sell_price = self.find_sell_highest_quantity(&lowest_kind, sell_exclude.clone());

        if buy_price < sell_price.0 * ONE_KIND_RESELLABLE_TREEHOLD {
            let quantity_price_option = self.calculate_amount_to_buy(
                market_index_lowest,
                lowest_kind,
                f32::INFINITY,
                percentage_of_wealth,
            );
            let quantity_price;
            match quantity_price_option {
                Some(quantity_price_option) => {
                    quantity_price = quantity_price_option;
                }
                None => {
                    return self.buy_all_of_kind_strat(percentage_of_wealth, sell_exclude);
                }
            }

            let token_buy = self.lock_buy(
                market_index_lowest,
                lowest_kind,
                quantity_price.0,
                quantity_price.1,
            );

            match token_buy {
                Ok(_) => {
                    self.buy(token_buy.unwrap()).ok();
                    return true;
                }
                Err(_) => {
                    /*
                    println!(
                        "BUYING {:?} FROM MARKET {} FOR {} EUR FAILED",
                        lowest_kind,
                        market_index_lowest,
                        quantity_price.0 * buy_price
                    );
                     */

                    return self.buy_all_of_kind_strat(percentage_of_wealth, sell_exclude);
                }
            }
        } else {
            /*
            println!(
                "not good enough price to buy {:?} from market {}, high sell: {:?}, buy :{}",
                lowest_kind, market_index_lowest, sell_price, buy_price
            );
             */

            return self.buy_all_of_kind_strat(percentage_of_wealth, sell_exclude);
        }
    }

    pub fn sell_all_strat(&mut self) -> bool {
        let mut vec_of_goods_to_sell = vec![];
        self.goods.iter().for_each(|(k, v)| {
            if *k != GoodKind::EUR && v.good.get_qty() > 0. {
                vec_of_goods_to_sell.push(*k);
                //println!("TRYING FORCED SELL ON: {:?}", k);
            }
        });

        let mut sold_something = false;

        vec_of_goods_to_sell.iter().for_each(|k| {
            if !self.sell_to_highest(*k, vec![], 1.) {
                //random float between 0.1 and 0.5
                let mut rng = rand::thread_rng();
                let random_f32: f32 = rng.gen_range(0.01..0.5);

                if self.sell_to_highest(*k, vec![], random_f32) {
                    sold_something = true;
                }
            } else {
                sold_something = true;
            }
        });

        if sold_something {
            self.end_game_stuck = 0;
        } else {
            self.end_game_stuck += 1;
            if self.end_game_stuck > 30 {
                self.done = true;
            }
        }

        return sold_something;
    }

    pub fn random_buy_strat(&mut self) -> bool {
        let goods = vec![GoodKind::USD, GoodKind::YUAN, GoodKind::YEN];
        let quantity_of_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();
        //randomly buy all kinds of goods
        let mut bought_something = false;

        goods.iter().for_each(|k| {
            let y: f32 = rand::thread_rng().gen();
            let v = (y * RANDOM_BUY_MIN_MAX_WEALTH_PERCENTAGE.0)
                + RANDOM_BUY_MIN_MAX_WEALTH_PERCENTAGE.1;

            let quantity_of_eur_to_spend = quantity_of_eur * v;

            if self.buy_random(*k, quantity_of_eur_to_spend, vec![]) {
                bought_something = true;
            }
        });

        return bought_something;
    }

    pub fn buy_low_sell_high_strat(&mut self, mut excluded_markets: Vec<usize>) -> bool {
        let rate_option = self.find_best_goodkind(excluded_markets.clone());
        let rate;
        if rate_option.is_none() {
            return false;
        } else {
            rate = rate_option.unwrap();
        }
        if rate.1 .2 > rate.1 .3 {
            let quantity_gain_to_sell_option = self.calculate_amount_to_sell(rate.1 .0, rate.0);
            let mut quantity_gain_to_sell;
            match quantity_gain_to_sell_option {
                Some(quantity_gain_to_sell_option) => {
                    quantity_gain_to_sell = quantity_gain_to_sell_option;
                }
                None => {
                    excluded_markets.push(rate.1 .0);
                    return self.buy_low_sell_high_strat(excluded_markets);
                }
            }

            let quantity_price_option =
                self.calculate_amount_to_buy(rate.1 .1, rate.0, quantity_gain_to_sell.0, 0.8);
            //println!("quantity_price: {:?}", quantity_price);
            let quantity_price;
            match quantity_price_option {
                Some(quantity_price_option) => {
                    quantity_price = quantity_price_option;
                }
                None => {
                    excluded_markets.push(rate.1 .1);
                    return self.buy_low_sell_high_strat(excluded_markets);
                }
            }

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

            if quantity > LOW_BUY_HIGH_SELL_MIN_QUANTITY
                && gain > price * LOW_BUY_HIGH_SELL_MIN_GAIN
                && price > 0.
            {
                let token_sell = self.lock_sell(rate.1 .0, rate.0, quantity, gain);
                let mut token_buy = self.lock_buy(rate.1 .1, rate.0, quantity, price);
                match token_sell {
                    Err(_) => {
                        excluded_markets.push(rate.1 .0);
                        return self.buy_low_sell_high_strat(excluded_markets);
                    }
                    _ => {
                        match token_buy {
                            Err(BidTooLow {
                                lowest_acceptable_bid: lowest,
                                ..
                            }) => {
                                //println!("Bid too low, lowest acceptable bid is: {}", lowest);
                                if gain > lowest {
                                    token_buy = self.lock_buy(rate.1 .1, rate.0, quantity, lowest);
                                    self.buy(token_buy.unwrap()).ok();
                                    self.sell(token_sell.unwrap()).ok();
                                    return true;
                                } else {
                                    excluded_markets.push(rate.1 .1);
                                    return self.buy_low_sell_high_strat(excluded_markets);
                                }
                            }
                            Err(_) => {
                                //panic
                                excluded_markets.push(rate.1 .1);
                                return self.buy_low_sell_high_strat(excluded_markets);
                            }
                            _ => {
                                self.buy(token_buy.unwrap()).ok();
                                self.sell(token_sell.unwrap()).ok();
                                return true;
                            }
                        }
                    }
                }

                /*

                //print all goods
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });

                 */
            } else {
                excluded_markets.push(rate.1 .0);
                return self.buy_low_sell_high_strat(excluded_markets);
            }

            //break;
        }
        return false;
    }

    pub fn buy_random(
        &mut self,
        good_kind: GoodKind,
        quantity_of_eur_to_spend: f32,
        mut exclude_markets: Vec<usize>,
    ) -> bool {
        if exclude_markets.len() == self.markets.len() {
            //println!("No market to buy from");
            return false;
        }

        let best_deal = self.find_buy_cheapest_quantity(&good_kind, exclude_markets.clone());
        let quantity = quantity_of_eur_to_spend / best_deal.0;
        let price = self.markets[best_deal.1]
            .borrow()
            .get_buy_price(good_kind, quantity);
        match price {
            Err(_) => {
                exclude_markets.push(best_deal.1);
                return self.buy_random(good_kind, quantity_of_eur_to_spend, exclude_markets);
            }
            _ => {
                let price = price.unwrap();
                if price * RANDOM_BUY_RESELLABLE_TREEHOLD > quantity_of_eur_to_spend {
                    let token_buy = self.lock_buy(best_deal.1, good_kind, quantity, price);
                    match token_buy {
                        Ok(_) => {
                            self.buy(token_buy.unwrap()).ok();
                            return true;
                        }
                        Err(_) => {
                            exclude_markets.push(best_deal.1);
                            return self.buy_random(
                                good_kind,
                                quantity_of_eur_to_spend,
                                exclude_markets,
                            );
                        }
                    }
                } else {
                    return false;
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
                return self.sell_to_highest(good_kind, excluded_markets, percentage);
            }
            _ => {
                if price.clone().unwrap() <= 0. {
                    excluded_markets.push(best_deal.1);
                    return self.sell_to_highest(good_kind, excluded_markets, percentage);
                }
                /*
                println!(
                    "PRice: {:?}, quantity: {}",
                    price.clone().unwrap(),
                    quantity
                );
                 */
                let token_sell = self.lock_sell(best_deal.1, good_kind, quantity, price.unwrap());
                match token_sell {
                    Err(_) => {
                        excluded_markets.push(best_deal.1);
                        return self.sell_to_highest(good_kind, excluded_markets, percentage);
                    }
                    _ => {
                        self.sell(token_sell.unwrap()).ok();
                    }
                }
            }
        }

        return true;
    }

    pub fn buy_low_sell_high_strat_day(&mut self) {
        if self.day < MAX_DAYS {
            let starting_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            self.buy_low_sell_high_strat(vec![]);

            let ending_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            if !(starting_eur < ending_eur) {
                if self.not_optimal_counter > NOT_OPTIMAL_TREEHOLD {
                    self.sell_all_strat();
                    self.not_optimal_counter = 0;
                }

                //println!("--------------------------------------------\nWAIT\n--------------------------------------------");
                self.wait_update();

                /*                //print all goods
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");
                 */

                self.not_optimal_counter += 1;
            } else {
                //print all goods
                /*
                print!("-----------------NOICE_______________________________________________________________________________________________________--------------------");
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");
                 */

                self.not_optimal_counter = 0;
            }
        } else if self.day >= MAX_DAYS && !self.done {
            self.end_game_sell_all();
        } else {
            self.wait_update();
        }
    }

    pub fn buy_all_of_kind_strat_day(&mut self) {
        if self.day < MAX_DAYS {
            if self.not_optimal_counter > NOT_OPTIMAL_TREEHOLD {
                self.sell_all_strat();
                self.not_optimal_counter = 0;
            }

            let mut rng = rand::thread_rng();
            let random_selector: f32 = rng.gen();
            if random_selector < ONE_KIND_PICK_PERCENTAGE {
                //println!("--------------------------------------------\nBUY ALL OF KIND\n--------------------------------------------");

                if !self.buy_all_of_kind_strat(ONE_KIND_WEALTH_PERCENTAGE, vec![]) {
                    self.wait_update();
                }
            } else {
                //println!("--------------------------------------------\nWAIT\n--------------------------------------------");
                self.wait_update();
            }

            self.not_optimal_counter += 1;
            /*
            //print all goods
            println!("------------------------------------");
            self.goods.iter().for_each(|(k, v)| {
                println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
            });
            println!("------------------------------------");
             */
        } else if self.day >= MAX_DAYS && !self.done {
            self.end_game_sell_all();
        } else {
            self.wait_update();
        }
    }

    pub fn random_buy_strat_day(&mut self) {
        if self.day < MAX_DAYS {
            if self.not_optimal_counter > NOT_OPTIMAL_TREEHOLD {
                self.sell_all_strat();
                self.not_optimal_counter = 0;
            } else {
                let mut rng = rand::thread_rng();
                let random_selector: f32 = rng.gen();
                if random_selector < RANDOM_BUY_PICK_PERCENTAGE {
                    //println!("--------------------------------------------\nBUY ALL OF KIND\n--------------------------------------------");

                    if !self.random_buy_strat() {
                        self.wait_update();
                    }
                } else {
                    //println!("--------------------------------------------\nWAIT\n--------------------------------------------");
                    self.wait_update();
                }
                self.not_optimal_counter += 1;
            }
            /*
            //print all goods
            println!("------------------------------------");
            self.goods.iter().for_each(|(k, v)| {
                println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
            });
            println!("------------------------------------");
             */
        } else if self.day >= MAX_DAYS && !self.done {
            self.end_game_sell_all();
        } else {
            self.wait_update();
        }
    }

    pub fn main_strat_day(&mut self) {
        if self.day < MAX_DAYS {
            let starting_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            self.buy_low_sell_high_strat(vec![]);

            let ending_eur = self.goods.get(&GoodKind::EUR).unwrap().good.get_qty();

            if !(starting_eur < ending_eur) {
                if self.not_optimal_counter > NOT_OPTIMAL_TREEHOLD {
                    self.sell_all_strat();
                    self.not_optimal_counter = 0;
                }

                let mut rng = rand::thread_rng();
                let random_sub_optimal_selector: f32 = rng.gen();
                if random_sub_optimal_selector < MAIN_RANDOM_BUY_PICK_PERCENTAGE {
                    //println!("--------------------------------------------\nBUY ALL OF KIND\n--------------------------------------------");

                    self.buy_all_of_kind_strat(MAIN_ONE_KIND_WEALTH_PERCENTAGE, vec![]);
                } else if random_sub_optimal_selector
                    < MAIN_RANDOM_BUY_PICK_PERCENTAGE + MAIN_ONE_KIND_PICK_PERCENTAGE
                {
                    //println!("--------------------------------------------\nRANDOM BUY\n--------------------------------------------");

                    self.random_buy_strat();
                } else {
                    //println!("--------------------------------------------\nWAIT\n--------------------------------------------");
                    self.wait_update();
                }

                /*
                //print all goods
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");
                */

                self.not_optimal_counter += 1;
            } else {
                //print!("-----------------NOICE_______________________________________________________________________________________________________--------------------");
                /*
                //print all goods
                println!("------------------------------------");
                self.goods.iter().for_each(|(k, v)| {
                    println!("Good: {:?}, quantity: {}", k, v.good.get_qty());
                });
                println!("------------------------------------");
                 */
                self.not_optimal_counter = 0;
            }
        } else if self.day >= MAX_DAYS && !self.done {
            self.end_game_sell_all();
        } else {
            self.wait_update();
        }
    }
}
