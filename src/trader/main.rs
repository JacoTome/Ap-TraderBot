use std::collections::VecDeque;
use std::sync::Mutex;

use crate::utils::market::{CurrencyData, DailyCurrencyData, MarketEvent, TraderTrait};
use crate::TypeMarket;
use unitn_market_2022::good::good_kind::GoodKind;
pub struct Trader<'a> {
    running: &'a Mutex<bool>,
    market: &'a Mutex<TypeMarket>,
    trader_data: &'a Mutex<VecDeque<DailyCurrencyData>>,
    selected_strategy: &'a Mutex<String>,
    trader: Box<dyn TraderTrait>,
    total_currency: CurrencyData,
}
const STRATEGIES: &'static [&'static str] = &[
    "Default", "Prova1", "Prova2", "Prova3", "Prova4", "Prova5", "Prova6",
]; // TODO: INSERT STRATEGIES HERE

impl<'a> Trader<'a> {
    pub fn new(
        run: &'a Mutex<bool>,
        market: &'a Mutex<TypeMarket>,
        trader_data: &'a Mutex<VecDeque<DailyCurrencyData>>,
        strategy: &'a Mutex<String>,
        trader: Box<dyn TraderTrait>,
    ) -> Self {
        let curr = trader.get_trader_data();
        Trader {
            running: run,
            market,
            trader_data,
            selected_strategy: strategy,
            trader: trader,
            total_currency: curr,
        }
    }

    fn get_strat_index(&self) -> i32 {
        let binding = self.selected_strategy.lock().unwrap();
        let mut index = 0;
        for (i, strat) in STRATEGIES.iter().enumerate() {
            if *strat == *binding {
                index = i as i32;
                break;
            }
        }
        index
    }

    pub fn is_running(&self) -> bool {
        match self.running.lock() {
            Ok(binding) => *binding,

            Err(e) => {
                println!("Error: {}", e);
                false
            }
        }
    }
    pub fn switch_run_pause(&mut self) {
        let mut binding = self.running.lock().unwrap();
        *binding = !*binding;
    }

    fn update_daily_data(&mut self) {
        let data = self.trader.get_daily_data();
        for data in data {
            let mut binding = self.trader_data.lock().unwrap();
            match data.event {
                MarketEvent::Buy | MarketEvent::Sell => {
                    let mut currencies = self.total_currency.clone();
                    match data.kind_given {
                        GoodKind::EUR => currencies.eur -= data.amount_given,
                        GoodKind::USD => currencies.usd -= data.amount_given,
                        GoodKind::YEN => currencies.yen -= data.amount_given,
                        GoodKind::YUAN => currencies.yuan -= data.amount_given,
                    }

                    match data.kind_received {
                        GoodKind::EUR => currencies.eur += data.amount_received,
                        GoodKind::USD => currencies.usd += data.amount_received,
                        GoodKind::YEN => currencies.yen += data.amount_received,
                        GoodKind::YUAN => currencies.yuan += data.amount_received,
                    }

                    let daily_data = DailyCurrencyData {
                        currencies: currencies.clone(),
                        daily_data: data.clone(),
                    };
                    binding.push_back(daily_data.clone());
                    self.total_currency = currencies;
                }
                _ => {
                    let daily_data = DailyCurrencyData {
                        currencies: self.total_currency.clone(),
                        daily_data: data.clone(),
                    };
                    binding.push_back(daily_data.clone());
                }
            }
        }
    }

    pub fn update_market(&mut self) {
        let mut market_data = self.trader.get_market_data();
        let mut binding = self.market.lock().unwrap();
        binding.append(&mut market_data);
    }

    pub fn pass_one_day(&mut self) {
        // Daily update
        self.trader.progess_day(self.get_strat_index());
        self.update_daily_data();
        self.update_market();
    }
}
