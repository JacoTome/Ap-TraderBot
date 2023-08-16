// pub mod trader;
use std::{borrow::BorrowMut, collections::HashMap, iter, result};

use csv;
use eframe::egui;
use egui::plot::{Line, PlotPoint, PlotPoints};
use itertools::Itertools;
// use trader::Trader;
// use unitn_market_2022::{good::good_kind::GoodKind, market::good_label::GoodLabel};
fn main() {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Trader visualization",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}

struct MyApp {
    goods: HashMap<String, Vec<f64>>,
    goods_to_show: HashMap<String, Vec<f64>>,
    // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl<'a> MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let goods = Self::read_from_file();
        let goods_to_show: HashMap<String, Vec<f64>> = HashMap::new();
        Self {
            goods,
            goods_to_show,
        }
    }

    fn load_one(&mut self) {
        // Push one good from self.goods in goods_to_show at a time
        for (key, value) in self.goods.iter_mut() {
            if value.len() == 0 {
                continue;
            } else {
                let val = value.remove(0);
                if self.goods_to_show.get(key).is_none() {
                    self.goods_to_show.insert(key.to_string(), vec![val]);
                } else {
                    self.goods_to_show.get_mut(key).unwrap().push(val);
                }
            }
        }
    }

    fn load_all(&mut self) {
        // Push all goods from self.goods in goods_to_show
        for (key, value) in self.goods.iter_mut() {
            if value.len() == 0 {
                continue;
            } else {
                self.goods_to_show.insert(key.to_string(), value.to_vec());
            }
        }
    }

    fn read_from_file() -> HashMap<String, Vec<f64>> {
        // Open csv file
        let mut rdr = csv::Reader::from_path("market.csv").unwrap();

        let mut ret_val = HashMap::new();
        // get headers
        {
            let headers = rdr.headers().unwrap();
            for header in headers {
                ret_val.insert(header.to_string(), Vec::new());
            }
        }
        let result_iter = rdr.records();

        // Iterate over records
        for result in result_iter {
            //Check if record is unwrappable
            if result.is_err() {
                continue;
            }
            let record = result.unwrap();
            // Insert in self.goods iterating on self.goods keys
            for (index, (_key, value)) in ret_val.iter_mut().enumerate() {
                // Parse record to f64
                if record[index].parse::<f64>().is_err() {
                    continue;
                } else {
                    let rec = record[index].parse::<f64>().unwrap();
                    value.push(rec);
                }
                // catch error
            }
        }

        ret_val
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Trader visualization");
                if ui.button("Read from file").clicked() {
                    self.load_all();
                }
                if ui.button("Read one by one").clicked() {
                    self.load_one();
                }
                ui.end_row();
            });
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Trader status");
            ui.end_row();
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.horizontal(|ui| {
                for (key, value) in self.goods.iter() {
                    if value.last().is_none() {
                        continue;
                    } else {
                        ui.label(format!("{}: {} || ", key, value.last().unwrap()));
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Budget");
            let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                for (keys, values) in self.goods_to_show.iter() {
                    let points = values
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| [i as f64, v])
                        .collect_vec();
                    ui.line(Line::new(PlotPoints::new(points)).name(keys));
                }
                // ui.line(Line::new(PlotPoints::new(
                //     self.trader.get_good_history(GoodKind::EUR),
                // )));
                // ui.line(Line::new(PlotPoints::new(
                //     self.trader.get_good_history(GoodKind::USD),
                // )));
                // ui.line(Line::new(PlotPoints::new(
                //     self.trader.get_good_history(GoodKind::YEN),
                // )));
                // ui.line(Line::new(PlotPoints::new(
                //     self.trader.get_good_history(GoodKind::YUAN),
                // )));
            });
        });
    }
}
