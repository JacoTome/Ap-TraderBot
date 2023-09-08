use super::Trade;
use super::TraderRicca;

use crate::utils::market::{DailyData, MarketEvent};
use unitn_market_2022::event::event::EventKind::{LockedBuy, LockedSell};
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, SellError};

impl TraderRicca {
    pub fn lock_buy(
        &mut self,
        market_index: usize,
        good_kind: GoodKind,
        quantity_to_buy: f32,
        price_to_pay: f32,
    ) -> Result<String, LockBuyError> {
        let event = self.markets[market_index].borrow_mut().lock_buy(
            good_kind,
            quantity_to_buy,
            price_to_pay,
            self.name.clone(),
        );

        match event {
            Ok(token) => {
                let trade = Trade {
                    trans_kind: LockedBuy,
                    kind_giving: GoodKind::EUR,
                    kind_receiving: good_kind,
                    giving_quantity: price_to_pay,
                    receiving_quantity: quantity_to_buy,
                    market_index: market_index,
                };
                self.traders_locked.insert(token.clone(), trade);

                self.last_daily_data.push(DailyData {
                    event: MarketEvent::LockBuy,
                    amount_given: price_to_pay as f64,
                    amount_received: quantity_to_buy as f64,
                    kind_given: GoodKind::EUR,
                    kind_received: good_kind,
                });
                self.add_market_event();

                self.day_passed();
                /*
                println!(
                    "{} locked buy {} {} for {} EUR on {}",
                    self.name, quantity_to_buy, good_kind, price_to_pay, market_index
                );
                println!("DAY NUMBER: {}", self.day);
                 */
                return Ok(token);
            }
            Err(e) => {
                //println!("on market {} Error: {:?}", market_index, e);
                return Err(e);
            }
        }
    }

    pub fn buy(&mut self, token: String) -> Result<(), BuyError> {
        let trade = self.traders_locked.get_mut(&*token).unwrap();
        if trade.giving_quantity > self.goods.get(&trade.kind_giving).unwrap().good.get_qty() {
            //println!("Not enough money");
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
                        .merge(good)
                        .ok();

                    self.last_daily_data.push(DailyData {
                        event: MarketEvent::Buy,
                        amount_given: trade.giving_quantity as f64,
                        amount_received: trade.receiving_quantity as f64,
                        kind_given: GoodKind::EUR,
                        kind_received: trade.kind_receiving,
                    });

                    /*
                    println!(
                        "{} bought {} {} for {} EUR on {}",
                        self.name,
                        trade.receiving_quantity,
                        trade.kind_receiving,
                        trade.giving_quantity,
                        trade.market_index
                    );
                     */

                    self.add_market_event();
                    self.traders_locked.remove(&*token);

                    self.day_passed();
                    //println!("DAY NUMBER: {}", self.day);

                    return Ok(());
                }
                Err(e) => {
                    match e {
                        BuyError::ExpiredToken { .. } => {
                            //println!("Expired token");
                            self.traders_locked.remove(&*token);
                        }
                        _ => {
                            //println!("Error: {:?}", e);
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
        good_kind: GoodKind,
        quantity_to_sell: f32,
        price_wanted: f32,
    ) -> Result<String, LockSellError> {
        let event = self.markets[market_index].borrow_mut().lock_sell(
            good_kind,
            quantity_to_sell,
            price_wanted,
            self.name.clone(),
        );
        match event {
            Ok(token) => {
                let trade = Trade {
                    trans_kind: LockedSell,
                    kind_giving: good_kind,
                    kind_receiving: GoodKind::EUR,
                    giving_quantity: quantity_to_sell,
                    receiving_quantity: price_wanted,
                    market_index: market_index,
                };

                self.last_daily_data.push(DailyData {
                    event: MarketEvent::LockSell,
                    amount_given: quantity_to_sell as f64,
                    amount_received: price_wanted as f64,
                    kind_given: good_kind,
                    kind_received: GoodKind::EUR,
                });
                self.add_market_event();

                self.traders_locked.insert(token.clone(), trade);
                self.day_passed();
                /*
                println!(
                    "{} locked sell {} {} for {} EUR on {}",
                    self.name, quantity_to_sell, good_kind, price_wanted, market_index
                );
                println!("DAY NUMBER: {}", self.day);
                 */
                return Ok(token);
            }
            Err(e) => {
                //println!("on market {} Error: {:?}", market_index, e);
                return Err(e);
            }
        }
    }

    pub fn sell(&mut self, token: String) -> Result<(), SellError> {
        let trade = self.traders_locked.get_mut(&*token).unwrap();
        if trade.giving_quantity > self.goods.get(&trade.kind_giving).unwrap().good.get_qty() {
            //println!("Not enough money");
            return Err(SellError::InsufficientGoodQuantity {
                contained_quantity: 0.0,
                pre_agreed_quantity: -1.0,
            });
        } else {
            let selling_good = self
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
                        .merge(good)
                        .ok();

                    self.last_daily_data.push(DailyData {
                        event: MarketEvent::Sell,
                        amount_given: trade.giving_quantity as f64,
                        amount_received: trade.receiving_quantity as f64,
                        kind_given: trade.kind_giving,
                        kind_received: GoodKind::EUR,
                    });

                    /*
                    println!(
                        "{} sold {} {} for {} EUR on {}",
                        self.name,
                        trade.giving_quantity,
                        trade.kind_giving,
                        trade.receiving_quantity,
                        trade.market_index
                    );
                    */

                    self.add_market_event();
                    self.traders_locked.remove(&*token);
                    self.day_passed();
                    //println!("DAY NUMBER: {}", self.day);
                    return Ok(());
                }
                Err(e) => {
                    match e {
                        SellError::ExpiredToken { .. } => {
                            //println!("Expired token");

                            self.goods
                                .get_mut(&trade.kind_giving)
                                .unwrap()
                                .good
                                .merge(selling_good)
                                .ok();
                            self.traders_locked.remove(&*token);
                        }
                        _ => {
                            //println!("Error: {:?}", e);
                            self.goods
                                .get_mut(&trade.kind_giving)
                                .unwrap()
                                .good
                                .merge(selling_good)
                                .ok();
                        }
                    }

                    return Err(e);
                }
            }
        }
    }
}
