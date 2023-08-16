// pub mod trader;
use std::collections::HashMap;
use std::io::Read;
use std::net;

use csv;
use eframe::egui;
use egui::plot::{BarChart, Line, PlotPoints};
use itertools::Itertools;
// use trader::Trader;
// use unitn_market_2022::{good::good_kind::GoodKind, market::good_label::GoodLabel};

fn _init_tcp_connection() {
    // Init tcp connection
    let mut trader_stream = net::TcpStream::connect("127.0.0.14:6969").unwrap();
    let mut buffer = [0; 1024];
    let size = trader_stream.read(&mut buffer).unwrap();
    let _msg = String::from_utf8_lossy(&buffer[..size]);
}

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
    graph_choose: usize,
    start: bool,
    load_value: usize,
    // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl<'a> MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let goods = Self::read_from_file();
        let goods_to_show: HashMap<String, Vec<f64>> = HashMap::new();
        let graph_choose = 0;
        let start = false;
        Self {
            goods,
            goods_to_show,
            graph_choose,
            start,
            load_value: 0,
        }
    }

    fn load_value(&mut self, tot: usize) {
        // Push tot good from self.goods in goods_to_show
        for (key, value) in self.goods.iter_mut() {
            if value.len() == 0 {
                continue;
            } else {
                if tot > value.len() {
                    println!("Not enough values in {}", key);
                    continue;
                }
                let val = value.drain(0..tot).collect_vec();
                if self.goods_to_show.get(key).is_none() {
                    self.goods_to_show.insert(key.to_string(), val);
                } else {
                    self.goods_to_show.get_mut(key).unwrap().extend(val);
                }
            }
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
                ui.label("Load how many?");
                let mut val_string = format!("{}", self.load_value);
                let value = ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut val_string));
                if value.changed() {
                    if !val_string.parse::<usize>().is_err() {
                        self.load_value = val_string.parse::<usize>().unwrap();
                    }
                }
                if ui.button("Load").clicked() || ui.input().key_pressed(egui::Key::Enter) {
                    self.load_value(self.load_value);
                }
            });
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Trader status");
            ui.end_row();
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.horizontal_wrapped(|ui| {
                for (key, value) in self.goods_to_show.iter() {
                    if value.last().is_none() {
                        continue;
                    } else {
                        ui.label(format!("{}: {} || ", key, value.last().unwrap()));
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Boh");
            ui.separator();
            // Top down menu
            ui.menu_button("Choose Graph", |ui| {
                ui.selectable_value(&mut self.graph_choose, 0, "Plot");
                ui.selectable_value(&mut self.graph_choose, 1, "Bars");
            });
            ui.separator();
            // if ui.button("Start/Stop").clicked() {
            //     self.start = !self.start;
            // }

            // if self.start {
            //     self.load_one();
            // }

            // Plot
            match self.graph_choose {
                0 => {
                    ui.label("Plot");
                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        for (keys, values) in self.goods_to_show.iter() {
                            let points = values
                                .iter()
                                .enumerate()
                                .map(|(i, &v)| [i as f64, v])
                                .collect_vec();
                            ui.line(Line::new(PlotPoints::new(points)).name(keys));
                        }
                    });
                }
                1 => {
                    ui.label("Bars");
                    let bars = self
                        .goods_to_show
                        .iter()
                        .enumerate()
                        .map(|(i, (key, value))| {
                            let mut bar =
                                egui::widgets::plot::Bar::new(i as f64, *value.last().unwrap());
                            bar = bar.name(key);
                            bar
                        })
                        .collect_vec();
                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        ui.bar_chart(BarChart::new(bars));
                    });
                }
                _ => {}
            }

            // let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
            //     for (keys, values) in self.goods_to_show.iter() {
            //         let points = values
            //             .iter()
            //             .enumerate()
            //             .map(|(i, &v)| [i as f64, v])
            //             .collect_vec();
            //         ui.line(Line::new(PlotPoints::new(points)).name(keys));
            //     }
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
    }
}
