pub mod trader;
use std::collections::HashMap;

use eframe::egui;
use egui::plot::{Line, PlotPoints};
use trader::Trader;
use unitn_market_2022::{good::good_kind::GoodKind, market::good_label::GoodLabel};
fn main() {
    tracing_subscriber::fmt::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Trader visualization",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}

struct MyApp<'a> {
    trader: Trader<'a>,
    market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl<'a> MyApp<'_> {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let trd = Trader::new();
        let market_goods = trd.get_market_goods();
        Self {
            trader: trd,
            market_goods: market_goods,
        }
    }
}

impl eframe::App for MyApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Trader visualization");
                if ui.button("Make transaction").clicked() {
                    self.trader.transaction();
                    self.market_goods = self.trader.get_market_goods();
                }
                ui.end_row();
            });
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Trader status");
            ui.label(self.trader.get_trader_status());
            ui.end_row();
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.horizontal(|ui| {
                for (market_name, goods) in &self.market_goods {
                    ui.vertical(|ui| {
                        ui.label(market_name);
                        for good in goods {
                            ui.label(format!(
                                "{}\n Quantity: {}\n Exchange rate buy: {}\n Exchange rate sell: {}",
                                good.good_kind.to_string(),
                                good.quantity,
                                good.exchange_rate_buy,
                                good.exchange_rate_sell
                            ));
                        }
                    });
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Budget");
            let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                ui.line(Line::new(PlotPoints::new(
                    self.trader.get_good_history(GoodKind::EUR),
                )));
                ui.line(Line::new(PlotPoints::new(
                    self.trader.get_good_history(GoodKind::USD),
                )));
                ui.line(Line::new(PlotPoints::new(
                    self.trader.get_good_history(GoodKind::YEN),
                )));
                ui.line(Line::new(PlotPoints::new(
                    self.trader.get_good_history(GoodKind::YUAN),
                )));
            });
        });
    }
}
