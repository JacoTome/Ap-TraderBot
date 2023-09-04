use crate::data_models::market::*;

pub fn compare_currencies(data1: &CurrencyData, data2: &CurrencyData) -> bool {
    data1.eur == data2.eur
        && data1.usd == data2.usd
        && data1.yen == data2.yen
        && data1.yuan == data2.yuan
}

pub fn compare_dailydata(data1: &DailyData, data2: &DailyData) -> bool {
    data1.event == data2.event
        && data1.kind_given == data2.kind_given
        && data1.kind_received == data2.kind_received
        && data1.amount_given == data2.amount_given
        && data1.amount_received == data2.amount_received
}

pub fn print_event(event: MarketEvent) -> String {
    match event {
        MarketEvent::Wait => "Wait".to_string(),
        MarketEvent::LockSell => "Lock Sell".to_string(),
        MarketEvent::LockBuy => "Lock Buy".to_string(),
        MarketEvent::Sell => "Sell".to_string(),
        MarketEvent::Buy => "Buy".to_string(),
    }
}

pub fn print_kind(kind: Currency) -> String {
    match kind {
        Currency::EUR => "EUR".to_string(),
        Currency::USD => "USD".to_string(),
        Currency::YEN => "YEN".to_string(),
        Currency::YUAN => "YUAN".to_string(),
    }
}

pub fn sum_currencies(data1: &CurrencyData, data2: &CurrencyData) -> CurrencyData {
    CurrencyData {
        eur: data1.eur + data2.eur,
        usd: data1.usd + data2.usd,
        yen: data1.yen + data2.yen,
        yuan: data1.yuan + data2.yuan,
    }
}
