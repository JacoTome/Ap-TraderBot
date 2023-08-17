use std::collections::HashMap;
use std::io::Read;
use std::net;

use csv;
use eframe::egui;
use egui::plot::{Bar, BarChart, Line, PlotPoints};
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
    goods_to_show: HashMap<String, (bool, Vec<f64>)>,
    graph_choose: usize,
    load_value: usize,
    // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl<'a> MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let goods = Self::read_from_file();
        let mut goods_to_show: HashMap<String, (bool, Vec<f64>)> = HashMap::new();
        let graph_choose = 0;

        for (key, _value) in goods.iter() {
            goods_to_show.insert(key.to_string(), (true, vec![0.0]));
        }
        Self {
            goods,
            goods_to_show,
            graph_choose,
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
                    self.goods_to_show.get_mut(key).unwrap().1.extend(val);
                } else {
                    self.goods_to_show.get_mut(key).unwrap().1.extend(val);
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
                    self.goods_to_show.get_mut(key).unwrap().1.push(val);
                } else {
                    self.goods_to_show.get_mut(key).unwrap().1.push(val);
                }
            }
        }
    }

    fn load_all(&mut self) {
        // Push all goods from self.goods in goods_to_show
        self.goods_to_show.clear();
        if !self.goods.is_empty() {
            for (key, value) in self.goods.iter_mut() {
                if value.len() == 0 {
                    continue;
                } else {
                    self.goods_to_show
                        .insert(key.to_string(), (true, value.to_vec()));
                }
            }
        }
        self.goods.clear();
    }

    fn read_from_file() -> HashMap<String, Vec<f64>> {
        // Open csv file
        let mut rdr = csv::Reader::from_path("../market.csv").unwrap();

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
                let load_all_button = ui.button("Load all");
                let load_one_button = ui.button("Load one");

                if load_all_button.clicked() {
                    self.load_all();
                }
                if load_one_button.clicked() {
                    self.load_one();
                }

                ui.end_row();
                ui.label("Load how many?");
                let mut val_string = format!("{}", self.load_value);

                let value = ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut val_string));
                let load_value = ui.button("Load");
                if value.changed() {
                    if !val_string.parse::<usize>().is_err() {
                        self.load_value = val_string.parse::<usize>().unwrap();
                    }
                }
                if load_value.clicked() || ui.input().key_pressed(egui::Key::Enter) {
                    self.load_value(self.load_value);
                }
            });
        });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Select goods to show");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (key, _value) in self.goods_to_show.clone().iter() {
                    ui.checkbox(&mut self.goods_to_show.get_mut(key).unwrap().0, key);
                }
            });
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.horizontal_wrapped(|ui| {
                for (key, value) in self.goods_to_show.iter() {
                    if value.1.last().is_none() {
                        continue;
                    } else {
                        ui.label(format!("{}: {} || \n ", key, value.1.last().unwrap()));
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

            // Plot
            match self.graph_choose {
                0 => {
                    ui.label("Plot");
                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        for (keys, values) in self.goods_to_show.iter() {
                            let points = values
                                .1
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
                    let values = self
                        .goods_to_show
                        .iter()
                        .map(|(key, value)| (key, (value.0, *value.1.last().unwrap())))
                        .collect_vec();
                    // Sort values
                    let values = values
                        .into_iter()
                        .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        .collect_vec();
                    let mut bars: Vec<Bar> = Vec::new();

                    for (i, bar_val) in values.iter().enumerate() {
                        if bar_val.1 .0 {
                            let mut bar = egui::widgets::plot::Bar::new(i as f64, bar_val.1 .1);
                            bar = bar.name(bar_val.0);
                            bar = bar
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(
                                        ((i * 40) % 255) as u8,
                                        ((i * 30) % 255) as u8,
                                        ((i * 20) % 255) as u8,
                                    ),
                                ))
                                .fill(egui::Color32::from_rgb(
                                    ((i * 40) % 255) as u8,
                                    ((i * 30) % 255) as u8,
                                    ((i * 20) % 255) as u8,
                                ));
                            bars.push(bar)
                        }
                    }

                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        ui.bar_chart(BarChart::new(bars));
                    });
                }
                _ => {
                    ui.label("How did you get here?");
                }
            }
        });
    }
}
