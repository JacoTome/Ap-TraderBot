#[macro_use]
extern crate lazy_static;

use core::time::Duration;
use csv;
use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, PlotPoints},
    Color32,
};
use itertools::Itertools;
use std::sync::RwLock;
use std::{collections::HashMap, thread, thread::JoinHandle};

lazy_static! {
    pub static ref START_TRADER: RwLock<bool> = false.into();
    pub static ref GOODS_TO_SHOW: RwLock<HashMap<String, (bool, Vec<f64>)>> =
        RwLock::new(HashMap::new());
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

fn insert(key: String, val: f64) {
    if GOODS_TO_SHOW.read().unwrap().get(&key).is_none() {
        GOODS_TO_SHOW
            .write()
            .unwrap()
            .insert(key.to_string(), (true, vec![0.0]));
    } else {
        GOODS_TO_SHOW
            .write()
            .unwrap()
            .get_mut(&key)
            .unwrap()
            .1
            .push(val);
    }
}

fn get_values() -> HashMap<String, (bool, Vec<f64>)> {
    match GOODS_TO_SHOW.read() {
        Ok(goods) => goods.clone(),
        Err(e) => {
            println!("Error: {}", e);
            HashMap::new()
        }
    }
}

struct MyApp {
    goods: HashMap<String, Vec<f64>>,
    goods_to_show: HashMap<String, (bool, Vec<f64>)>,
    graph_choose: usize,
    load_value: usize,
    trader_thread: Option<JoinHandle<()>>,
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
            trader_thread: None,
        }
    }

    fn init_values(&mut self) {
        // Init values
        self.goods = Self::read_from_file();
        self.goods_to_show.clear();
        for (key, _value) in self.goods.iter().sorted_by_key(|x| x.0.to_lowercase()) {
            self.goods_to_show
                .insert(key.to_string(), (true, vec![0.0]));
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
        if !self.goods.is_empty() {
            self.goods_to_show.clear();
            for (key, value) in self.goods.iter_mut() {
                if value.len() == 0 {
                    continue;
                } else {
                    self.goods_to_show
                        .insert(key.to_string(), (true, value.to_vec()));
                }
            }
            self.goods.clear();
        }
    }

    fn set_show(&mut self, kind: i32) {
        match kind {
            0 => {
                for good in self.goods_to_show.iter_mut() {
                    if good.0.to_lowercase().contains("val") {
                        good.1 .0 = true;
                    } else {
                        good.1 .0 = false;
                    }
                }
            }
            1 => {
                for good in self.goods_to_show.iter_mut() {
                    if good.0.to_lowercase().contains("price") {
                        good.1 .0 = true;
                    } else {
                        good.1 .0 = false;
                    }
                }
            }
            2 => {
                for good in self.goods_to_show.iter_mut() {
                    if !good.0.to_lowercase().contains("val")
                        && !good.0.to_lowercase().contains("price")
                    {
                        good.1 .0 = true;
                    } else {
                        good.1 .0 = false;
                    }
                }
            }

            _ => {}
        }
    }

    fn read_from_file() -> HashMap<String, Vec<f64>> {
        // Open csv file
        let mut rdr = csv::Reader::from_path("market.csv").unwrap();

        let mut ret_val = HashMap::new();
        // get headers
        {
            let headers = rdr.headers().unwrap();
            let mut hed_vals = Vec::<String>::new();
            for header in headers {
                if !(header.to_lowercase().contains("iteration")
                    || header.to_lowercase().contains("mean")
                    || header == "")
                {
                    hed_vals.push(header.to_string());
                }
            }
            hed_vals.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            for header in hed_vals {
                ret_val.insert(header, Vec::new());
            }
        }

        // Iterate over records
        let result_iter = rdr.records();
        for result in result_iter {
            //Check if record is unwrappable
            if !result.is_err() {
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
                }
            }
        }

        ret_val
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(100));

        /*****************
           TOP PANEL
        ******************/
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(50.0);
                ui.label(
                    egui::RichText::new("Trader visualization")
                        .size(15.0)
                        .color(Color32::YELLOW),
                );

                let load_all_button = ui.button("Load all").on_hover_text("Load all values");
                if load_all_button.clicked() {
                    self.load_all();
                }

                let load_one_button = ui.button("Load one").on_hover_text("Load one value");
                if load_one_button.clicked() {
                    self.load_one();
                }

                ui.label("Load how many?");
                let mut val_string = format!("{}", self.load_value);
                let value = ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut val_string));
                let load_value = ui.button("Load");
                if value.changed() {
                    if val_string.parse::<usize>().is_ok() {
                        self.load_value = val_string.parse::<usize>().unwrap();
                    }
                }
                if load_value.clicked() || ui.input().key_pressed(egui::Key::Enter) {
                    self.load_value(self.load_value);
                }

                let start_button = ui.button("Start/Stop");
                if start_button.clicked() {
                    let val = START_TRADER.read().unwrap().to_owned();
                    *START_TRADER.write().unwrap() = !val;
                    if START_TRADER.read().unwrap().to_owned() {
                        if self.trader_thread.is_none() {
                            self.trader_thread = Some(thread::spawn(move || {
                                /********************
                                       TRADER
                                ********************/
                                let mut i = 0;
                                println!("Trader started");
                                loop {
                                    if START_TRADER.read().unwrap().to_owned() {
                                        i += 1;
                                        insert(
                                            "EUR".to_string(),
                                            rand::random::<f64>() * 100000 as f64,
                                        );
                                        thread::sleep(std::time::Duration::from_millis(100));
                                    } else {
                                        println!("Parked");
                                        thread::park();
                                    }
                                }
                            }));
                        } else {
                            self.trader_thread.as_ref().unwrap().thread().unpark();
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let reset_button = ui.button("Reset");
                    if reset_button.clicked() {
                        self.init_values();
                    }
                })
                // Reset button at the end of the row
            });
        });

        /*****************
           LEFT SIDE PANEL
        ******************/

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Select goods to show");
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    let select_all_button = ui.button("Select all");
                    if select_all_button.clicked() {
                        for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = true;
                        }
                    }
                    let deselect_all_button = ui.button("Deselect all");
                    if deselect_all_button.clicked() {
                        for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = false;
                        }
                    }
                });
                ui.separator();
                let mut kind_to_show = 0;
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.menu_button("Select kind", |ui| {
                        if ui.selectable_value(&mut kind_to_show, 0, "Val").clicked()
                            || ui.selectable_value(&mut kind_to_show, 1, "Price").clicked()
                            || ui
                                .selectable_value(&mut kind_to_show, 2, "Quantity")
                                .clicked()
                        {
                            self.set_show(kind_to_show);
                        }
                    });
                });

                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    for (key, _value) in self
                        .goods_to_show
                        .clone()
                        .iter()
                        .sorted_by_key(|x| x.0.to_lowercase())
                    {
                        ui.checkbox(&mut self.goods_to_show.get_mut(key).unwrap().0, key);
                    }
                })
            });
        });

        /*****************
           BOTTOM PANEL
        ******************/

        egui::SidePanel::right("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.set_min_width(150.0);
            ui.vertical(|ui| {
                for (key, value) in self
                    .goods_to_show
                    .iter()
                    .sorted_by_key(|x| x.0.to_lowercase())
                {
                    if !value.1.last().is_none() {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                                ui.set_min_width(50.0);
                                ui.label(key);
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                                ui.label(format!("{:.2}", value.1.last().unwrap()));
                            });
                        });
                    }
                }
            });
        });

        /*****************
           CENTRAL PANEL
        ******************/

        egui::CentralPanel::default().show(ctx, |ui| {
            // Top down menu
            ui.menu_button("Choose Graph", |ui| {
                ui.selectable_value(&mut self.graph_choose, 0, "Plot");
                ui.selectable_value(&mut self.graph_choose, 1, "Bars");
            });
            ui.separator();

            match self.graph_choose {
                0 => {
                    let tmp = get_values();
                    ui.label("Plot");
                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        for (i, (keys, values)) in tmp.iter().enumerate() {
                            if values.0 {
                                let points = values
                                    .1
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &v)| [i as f64, v])
                                    .collect_vec();
                                ui.line(Line::new(PlotPoints::new(points)).name(keys).color(
                                    egui::Color32::from_rgb(
                                        ((i + 5 * 1000) % 255) as u8,
                                        ((i + 4 * 900) % 255) as u8,
                                        ((i + 3 * 800) % 255) as u8,
                                    ),
                                ));
                            }
                        }
                    });
                }
                1 => {
                    ui.label("Bars");
                    /*
                    let values = self
                        .goods_to_show
                        .iter()
                        .map(|(key, value)| (key, (value.0, *value.1.last().unwrap())))
                        .collect_vec();
                    */
                    let tmp = get_values();
                    let values = tmp
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
