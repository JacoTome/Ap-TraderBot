pub struct DailyData {
    pub event: Event,
    pub amount_given: f64,
    pub amount_received: f64,
    pub kind_given: Currencies,
    pub kind_received: Currencies,
}

pub enum Event {
    Wait,
    LockSell,
    LockBuy,
    Sell,
    Buy,
}

pub enum Currencies {
    Eur,
    Usd,
    Yen,
    Yuan,
}

pub trait TraderEvents {
    fn daily_event(&self);
}

#[derive(Clone)]
pub struct MarketData {
    pub name: String,
    pub eur: f64,
    pub usd: f64,
    pub yen: f64,
    pub yuan: f64,
}

pub trait MarketEvents {
    fn update_data(&mut self, markets: &mut Vec<MarketData>);
}
