use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bose::market::BoseMarket;
use tracing::{error, info};
use unitn_market_2022::{
    good::{good::Good, good_kind::GoodKind},
    market::{good_label::GoodLabel, Market},
    subscribe_each_other, wait_one_day,
};

/*
    Trader objective:
    Starting with a budget of 10000 EUR, the trader has to sell EUR in order to buy the maximum amount of USD, YEN and YUAN.
*/

type KindHistory = HashMap<GoodKind, Vec<[f64; 2]>>;
pub struct Trader<'a> {
    pub name: String,
    pub budget: f32,
    pub budget_history: Vec<[f64; 2]>,
    pub goods: HashMap<GoodKind, Good>,
    pub good_history: KindHistory,
    pub locked_goods_buy: HashMap<GoodKind, Vec<String>>,
    market: HashMap<&'a str, Rc<RefCell<dyn Market>>>,
}

impl<'a> Trader<'a> {
    pub fn new() -> Self {
        let mut goods = HashMap::<GoodKind, Good>::new();
        goods.insert(GoodKind::EUR, Good::new(GoodKind::EUR, 9999.0));
        goods.insert(GoodKind::USD, Good::new(GoodKind::USD, 0.0));
        goods.insert(GoodKind::YEN, Good::new(GoodKind::YEN, 0.0));
        goods.insert(GoodKind::YUAN, Good::new(GoodKind::YUAN, 0.0));

        let mut market = HashMap::new();
        let bose = BoseMarket::new_random();
        let rcnz = rcnz_market::rcnz::RCNZ::new_random();
        let bvc = BVC::BVCMarket::new_random();

        market.insert(bose.borrow().get_name(), bose.clone());
        market.insert(rcnz.borrow().get_name(), rcnz.clone());
        market.insert(bvc.borrow().get_name(), bvc.clone());

        subscribe_each_other!(&market);

        let tot_budget = goods.iter().map(|(_, good)| good.get_qty()).sum();

        let mut good_history = HashMap::<GoodKind, Vec<[f64; 2]>>::new();
        for (kind, good) in &goods {
            good_history.insert(*kind, vec![[0.0, good.get_qty() as f64]]);
        }

        Self {
            name: "Trader".to_string(),
            budget: tot_budget,
            budget_history: vec![[0.0, tot_budget as f64]],
            goods: goods,
            market: market,
            locked_goods_buy: HashMap::<GoodKind, Vec<String>>::new(),
            good_history,
        }
    }

    fn update_goods(&mut self, kind_to_change: GoodKind, quantity: f32) {
        for (kind, good) in &mut self.goods {
            if kind == &kind_to_change {
                if quantity < 0.0 {
                    good.split(quantity.abs()).expect("Error splitting good");
                } else {
                    good.merge(Good::new(kind_to_change, quantity))
                        .expect("Error merging good");
                }
            } else {
                good.merge(Good::new(*kind, 0.0))
                    .expect("Error merging good");
            }
        }
    }

    fn update_goods_history(&mut self) {
        for (kind, history) in &mut self.good_history {
            history.push([
                history.len() as f64,
                self.goods.get(kind).unwrap().get_qty() as f64,
            ]);
        }
    }

    fn update_budget_history(&mut self) {
        self.budget_history
            .push([self.budget_history.len() as f64, self.budget as f64]);
    }

    pub fn transaction(&mut self) {
        /*
            1. For each USD, YEN,YUAN, find the best exchange rate for buy;


        */

        // 1. For each USD, YEN,YUAN, find the best exchange rate for buy;
        let eur = self.goods.get(&GoodKind::EUR).unwrap();
        let mut best_buy = HashMap::<GoodKind, (&str, f32)>::new();
        let mut wait = false;
        for (name, market) in &self.market {
            let market = market.borrow();
            for (kind, _) in &self.goods {
                if *kind == GoodKind::EUR {
                    continue;
                }
                let (seller, price) = (
                    market.get_name(),
                    market.get_buy_price(*kind, eur.get_qty() / 10.0),
                );

                match price {
                    Ok(price) => {
                        if let Some((_, best_price)) = best_buy.get(kind) {
                            if price < *best_price && price < eur.get_qty() / 3.0 {
                                best_buy.insert(*kind, (seller, price));
                            }
                        } else {
                            if price < eur.get_qty() / 3.0 {
                                best_buy.insert(*kind, (seller, price));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error in market {}: {:?}", name, e);
                    }
                }
            }
            // If i have visited all market and i couldn't find a good to buy, wait one day
            // don't think that this is the best solution but it's a start
            if best_buy.len() == 0 {
                wait = true;
            }
        }

        if wait {
            for (_, market) in &self.market {
                wait_one_day!(market);
            }
        }
        info!("Best buy: {:?}", best_buy);

        // 2. Lock the best buy for the most expensive good
        let most_expensive = best_buy
            .iter()
            .max_by(|(_, (_, price1)), (_, (_, price2))| price1.partial_cmp(price2).unwrap())
            .unwrap()
            .0;
        let (seller, price) = best_buy.get(most_expensive).unwrap();
        let seller = self.market.get(seller).unwrap();
        match seller.borrow_mut().lock_buy(
            *most_expensive,
            eur.get_qty() / 10.0,
            *price,
            self.name.clone(),
        ) {
            Ok(token) => {
                info!("Locked buy: {:?}", token);
                self.locked_goods_buy
                    .entry(*most_expensive)
                    .or_insert(vec![])
                    .push(token);
            }
            Err(e) => {
                error!("Cannot lock buy: {:?}", e);
            }
        }

        // buy
        let token = self
            .locked_goods_buy
            .get_mut(most_expensive)
            .unwrap()
            .pop()
            .unwrap()
            .clone();
        let mut good_to_pay = Good::new(GoodKind::EUR, *price);

        let buy = seller.borrow_mut().buy(token, &mut good_to_pay);
        match buy {
            Ok(good) => {
                info!("Buy: {:?} with price: {:?}", good, price);
                self.update_goods(good.get_kind(), good.get_qty());
                self.update_goods(good_to_pay.get_kind(), -*price);
                self.update_goods_history();
                self.update_budget_history();
            }
            Err(e) => {
                error!("Cannot buy: {:?}", e);
            }
        }
    }

    pub fn get_trader_status(&self) -> String {
        let mut status = String::new();
        status.push_str(&format!("Budget: {}\n", self.budget));
        for (kind, good) in &self.goods {
            status.push_str(&format!("{}: {}\n", kind.to_string(), good.get_qty()));
        }
        status
    }

    pub fn get_market_goods(&self) -> HashMap<String, Vec<GoodLabel>> {
        let mut market_goods = HashMap::<String, Vec<GoodLabel>>::new();
        for market in &self.market {
            let market = market.1.borrow();
            let goods = market.get_goods();
            market_goods.insert(market.get_name().to_string(), goods);
        }
        market_goods
    }
    pub fn get_budget_history(&self) -> Vec<[f64; 2]> {
        self.budget_history.clone()
    }

    pub fn get_good_history(&self, kind: GoodKind) -> Vec<[f64; 2]> {
        self.good_history.get(&kind).unwrap().clone()
    }
}
