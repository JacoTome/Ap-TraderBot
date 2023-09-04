use std::collections::VecDeque;
use std::sync::Mutex;

use crate::data_models::market::{
    Currency, CurrencyData, DailyCurrencyData, DailyData, MarketEvent, TraderTrait,
};
use crate::TypeMarket;
pub struct Trader<'a> {
    RUNNING: &'a Mutex<bool>,
    PAUSED: &'a Mutex<bool>,
    MARKET: &'a Mutex<TypeMarket>,
    TRADER_DATA: &'a Mutex<VecDeque<DailyCurrencyData>>,
    SELECTED_STRATEGY: &'a Mutex<String>,
    traders: Vec<Box<dyn TraderTrait>>,
    last_currency_data: DailyCurrencyData,
}
const STRATEGIES: &'static [&'static str] = &[
    "Default", "Prova1", "Prova2", "Prova3", "Prova4", "Prova5", "Prova6",
]; // TODO: INSERT STRATEGIES HERE

impl<'a> Trader<'a> {
    pub fn new(
        run: &'a Mutex<bool>,
        pause: &'a Mutex<bool>,
        market: &'a Mutex<TypeMarket>,
        trader_data: &'a Mutex<VecDeque<DailyCurrencyData>>,
        strategy: &'a Mutex<String>,
    ) -> Self {
        // Init trader data
        let mut traders = Vec::new();
        let mut trader = Box::new(crate::trader::trader_ricca::TraderRicca::initialize_trader(
            0,
        ));

        traders.push(trader);
        let daily_data = DailyCurrencyData {
            currencies: CurrencyData {
                eur: 100.0,
                usd: 100.0,
                yen: 100.0,
                yuan: 100.0,
            },
            daily_data: DailyData {
                event: MarketEvent::Wait,
                amount_given: 0.0,
                amount_received: 0.0,
                kind_given: Currency::EUR,
                kind_received: Currency::EUR,
            },
        };
        match trader_data.lock() {
            Ok(mut binding) => {
                binding.push_back(daily_data.clone());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        Trader {
            RUNNING: run,
            PAUSED: pause,
            MARKET: market,
            TRADER_DATA: trader_data,
            SELECTED_STRATEGY: strategy,
            traders: traders,
            last_currency_data: daily_data,
        }
    }

    fn daily_update(&mut self) {
        let mut binding = self.TRADER_DATA.lock().unwrap();
        let currencies_opt = Some(self.last_currency_data.clone());
        let mut currencies: CurrencyData;
        match currencies_opt {
            Some(c) => {
                currencies = c.currencies.clone();

                match data.event {
                    MarketEvent::Buy | MarketEvent::Sell => {
                        match data.kind_given {
                            Currency::EUR => currencies.eur -= data.amount_given,
                            Currency::USD => currencies.usd -= data.amount_given,
                            Currency::YEN => currencies.yen -= data.amount_given,
                            Currency::YUAN => currencies.yuan -= data.amount_given,
                        }

                        match data.kind_received {
                            Currency::EUR => currencies.eur += data.amount_received,
                            Currency::USD => currencies.usd += data.amount_received,
                            Currency::YEN => currencies.yen += data.amount_received,
                            Currency::YUAN => currencies.yuan += data.amount_received,
                        }

                        let daily_data = DailyCurrencyData {
                            currencies: currencies.clone(),
                            daily_data: data.clone(),
                        };
                        binding.push_back(daily_data.clone());
                        self.last_currency_data = daily_data.clone();
                    }
                    _ => {
                        println!("No buy or sell event");
                    }
                }
            }
            None => {
                println!("No data");
                let mut currencies = CurrencyData {
                    eur: 0.0,
                    usd: 0.0,
                    yen: 0.0,
                    yuan: 0.0,
                };
                match data.event {
                    MarketEvent::Buy | MarketEvent::Sell => {
                        match data.kind_given {
                            Currency::EUR => currencies.eur -= data.amount_given,
                            Currency::USD => currencies.usd -= data.amount_given,
                            Currency::YEN => currencies.yen -= data.amount_given,
                            Currency::YUAN => currencies.yuan -= data.amount_given,
                        }

                        match data.kind_received {
                            Currency::EUR => currencies.eur += data.amount_received,
                            Currency::USD => currencies.usd += data.amount_received,
                            Currency::YEN => currencies.yen += data.amount_received,
                            Currency::YUAN => currencies.yuan += data.amount_received,
                        }

                        let daily_data = DailyCurrencyData {
                            currencies: currencies.clone(),
                            daily_data: data.clone(),
                        };
                        binding.push_back(daily_data.clone());
                    }
                    _ => {
                        println!("No buy or sell event");
                    }
                }
            }
        };
    }
    fn update_daily_data(&mut self) {
        let new_data = DailyCurrencyData {
            currencies: self.traders[0].get_trader_data(),
            daily_data: self.traders[0].get_daily_data(),
        };
        let mut binding = self.TRADER_DATA.lock().unwrap();
        binding.push_back(new_data.clone());
    }

    pub fn get_strategies(&self) -> Vec<String> {
        STRATEGIES.iter().map(|s| s.to_string()).collect()
    }

    pub fn pass_one_day(&mut self) {
        // Daily update
        for trader in self.traders.iter_mut() {
            trader.progess_day();
            self.daily_update()
        }
    }
}
