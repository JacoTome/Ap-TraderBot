use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Copy)]
pub struct CurrencyData {
    pub eur: f64,
    pub usd: f64,
    pub yen: f64,
    pub yuan: f64,
}

impl Clone for CurrencyData {
    fn clone(&self) -> Self {
        CurrencyData {
            eur: self.eur,
            usd: self.usd,
            yen: self.yen,
            yuan: self.yuan,
        }
    }
}

impl Serialize for CurrencyData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Market", 4)?;
        state.serialize_field("eur", &self.eur)?;
        state.serialize_field("usd", &self.usd)?;
        state.serialize_field("yen", &self.yen)?;
        state.serialize_field("yuan", &self.yuan)?;
        state.end()
    }
}

pub struct MarketData {
    pub name: String,
    pub currencies: CurrencyData,
}

impl Clone for MarketData {
    fn clone(&self) -> Self {
        MarketData {
            name: self.name.clone(),
            currencies: self.currencies.clone(),
        }
    }
}

impl Serialize for MarketData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Market", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("currencies", &self.currencies)?;
        state.end()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    EUR,
    USD,
    YEN,
    YUAN,
}

#[derive(Serialize, Clone, Copy, PartialEq)]
pub enum MarketEvent {
    Wait,
    LockSell,
    LockBuy,
    Sell,
    Buy,
}

pub struct DailyData {
    pub event: MarketEvent,
    pub amount_given: f64,
    pub amount_received: f64,
    pub kind_given: GoodKind,
    pub kind_received: GoodKind,
}

impl Serialize for DailyData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DailyData", 5)?;
        state.serialize_field("event", &self.event)?;
        state.serialize_field("amount_given", &self.amount_given)?;
        state.serialize_field("amount_received", &self.amount_received)?;
        state.serialize_field("kind_given", &self.kind_given)?;
        state.serialize_field("kind_received", &self.kind_received)?;
        state.end()
    }
}

impl Clone for DailyData {
    fn clone(&self) -> Self {
        DailyData {
            event: self.event.clone(),
            amount_given: self.amount_given,
            amount_received: self.amount_received,
            kind_given: self.kind_given.clone(),
            kind_received: self.kind_received.clone(),
        }
    }
}

pub struct DailyCurrencyData {
    pub currencies: CurrencyData,
    pub daily_data: DailyData,
}

impl Serialize for DailyCurrencyData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DailyCurrencyData", 2)?;
        state.serialize_field("currencies", &self.currencies)?;
        state.serialize_field("daily_data", &self.daily_data)?;
        state.end()
    }
}

impl Clone for DailyCurrencyData {
    fn clone(&self) -> Self {
        DailyCurrencyData {
            currencies: self.currencies.clone(),
            daily_data: self.daily_data.clone(),
        }
    }
}

use std::vec;

pub trait TraderTrait {
    fn initialize_trader(stratIndex: i32) -> Self
    where
        Self: Sized;
    fn progess_day(&mut self);
    fn get_daily_data(&self) -> vec::Vec<DailyData>;
    fn get_trader_data(&self) -> CurrencyData;
    fn get_market_data(&self) -> vec::Vec<vec::Vec<MarketData>>;
}
