pub mod data_models;
pub mod trader;
pub mod utils;

#[macro_use]
extern crate lazy_static;
use core::time::Duration;
use csv;
use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, PlotPoints},
    Color32,
};
use itertools::{enumerate, Itertools};
use std::{
    collections::vec_deque::VecDeque, collections::HashMap, sync::Mutex, thread, thread::JoinHandle,
};

use utils::utils::{
    compare_currencies, compare_dailydata, print_event, print_kind, sum_currencies,
};

use crate::trader::main::Trader;
use data_models::market::{
    Currency, CurrencyData, DailyCurrencyData, DailyData, MarketData, MarketEvent,
};
const STRATEGIES: &'static [&'static str] = &[
    "Default", "Prova1", "Prova2", "Prova3", "Prova4", "Prova5", "Prova6",
]; // TODO: INSERT STRATEGIES HERE
pub type TypeMarket = HashMap<String, Vec<CurrencyData>>;

lazy_static! {
    pub static ref RUNNING: Mutex<bool> = Mutex::new(false);
    pub static ref MARKETS: Mutex<TypeMarket> = Mutex::new(HashMap::new());
    pub static ref TRADER_DATA: Mutex<VecDeque<DailyCurrencyData>> = Mutex::new(VecDeque::new());
    pub static ref SELECTED_STRATEGY: Mutex<String> = Mutex::new(STRATEGIES[0].to_string());
}

fn is_running() -> bool {
    *RUNNING.lock().unwrap()
}

fn switch_run_pause() {
    let val = *RUNNING.lock().unwrap();
    *RUNNING.lock().unwrap() = !val;
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
    goods_to_show: (Vec<DailyData>, Vec<CurrencyData>),
    total_currency: CurrencyData,
    graph_choose: usize,
    load_value: usize,
    trader_thread: Option<JoinHandle<()>>,
    history_daily_data: VecDeque<DailyData>,
    history_currencies: VecDeque<CurrencyData>,
    index: (bool, usize), // 0 = unchanged, 1 = changed
    index_max: usize,
    // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl<'a> MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let graph_choose = 0;

        let goods_to_show = (vec![], vec![]);

        Self {
            goods_to_show,
            total_currency: CurrencyData {
                eur: 0.0,
                usd: 0.0,
                yen: 0.0,
                yuan: 0.0,
            },
            graph_choose,
            load_value: 0,
            trader_thread: None,
            history_daily_data: VecDeque::new(),
            history_currencies: VecDeque::new(),
            index: (true, 0),
            index_max: 0,
        }
    }

    fn load_value(&mut self, tot: usize) {
        // Push tot good from self.goods in goods_to_show
        /*for (key, value) in self.goods.iter_mut() {
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
        }*/
    }

    fn load_one(&mut self) {
        // Push one good from self.goods in goods_to_show at a time
        /* for (key, value) in self.goods.iter_mut() {
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
        }*/
    }
    fn load_all(&mut self) {
        // Push all goods from self.goods in goods_to_show
        /* if !self.goods.is_empty() {
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
        }*/
    }

    fn set_show(&mut self, kind: i32) {
        /* match kind {
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
        }*/
    }

    fn _read_from_file() -> HashMap<String, Vec<f64>> {
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

    fn get_actual_index(&self) -> usize {
        if self.index.0 {
            if self.goods_to_show.1.len() == 0 {
                return 0;
            }
            self.goods_to_show.1.len() - 1
        } else {
            self.index.1 - 1
        }
    }

    fn minus_one_day(&mut self) {
        if self.index.1 > 0 {
            self.index.0 = false;
            self.index.1 -= 1;
            println!("Index: {}", self.index.1)
        }
    }
    fn plus_one_day(&mut self) {
        if self.index.1 < self.index_max {
            self.index.0 = false;
            self.index.1 += 1;
        }
    }

    fn get_values(&mut self) {
        if self.history_currencies.is_empty() || self.history_daily_data.is_empty() {
            self.update_data();
            return;
        } else {
            self.goods_to_show
                .0
                .push(self.history_daily_data.pop_front().unwrap());
            self.goods_to_show
                .1
                .push(self.history_currencies.pop_front().unwrap());
            self.index_max = self.goods_to_show.1.len().max(self.goods_to_show.0.len());
            if self.index.0 {
                self.index.1 = self.index_max;
            }
        }
        // match GOODS_TO_SHOW.read() {
        //     Ok(goods) => goods.clone(),
        //     Err(e) => {
        //         println!("Error: {}", e);
        //         HashMap::new()
        //     }
        // }
    }
    fn update_data(&mut self) {
        match TRADER_DATA.lock() {
            Ok(mut data) => {
                if data.is_empty() {
                    return;
                }
                let new_data = data.pop_front().unwrap().clone();

                self.total_currency = sum_currencies(&self.total_currency, &new_data.currencies);
                self.history_currencies
                    .push_back(new_data.currencies.clone());
                self.history_daily_data
                    .push_back(new_data.daily_data.clone());
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        }
    }

    fn top_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(50.0);
                ui.label(
                    egui::RichText::new("Trader visualization")
                        .size(15.0)
                        .color(Color32::YELLOW),
                );

                let load_all_button = ui.button("-1 Day").on_hover_text("Load all values");
                if load_all_button.clicked() {
                    self.minus_one_day();
                }

                let load_one_button = ui.button("+1 Day").on_hover_text("Load one value");
                if load_one_button.clicked() {
                    self.plus_one_day();
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
                    switch_run_pause();
                    self.index.0 = true;
                    if is_running() {
                        if self.trader_thread.is_none() {
                            self.trader_thread = Some(thread::spawn(move || {
                                /********************
                                       TRADER
                                ********************/
                                let paused = Mutex::new(false);
                                let mut trader = Trader::new(
                                    &RUNNING,
                                    &paused,
                                    &MARKETS,
                                    &TRADER_DATA,
                                    &SELECTED_STRATEGY,
                                );
                                println!("Trader started");
                                loop {
                                    if is_running() {
                                        trader.pass_one_day();
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
                        self.index.1 = self.index_max;
                        self.index.0 = false;
                    }
                })
                // Reset button at the end of the row
            });
        })
    }

    fn left_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Select goods to show");
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    let select_all_button = ui.button("Select all");
                    if select_all_button.clicked() {
                        /* for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = true;
                        }*/
                    }
                    let deselect_all_button = ui.button("Deselect all");
                    if deselect_all_button.clicked() {
                        /*    for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = false;
                        }*/
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
                    /* for (key, _value) in self
                        .goods_to_show
                        .clone()
                        .iter()
                        .sorted_by_key(|x| x.0.to_lowercase())
                    {
                        ui.checkbox(&mut self.goods_to_show.get_mut(key).unwrap().0, key);
                    }*/
                })
            });
        })
    }

    fn right_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        self.get_values();
        egui::SidePanel::right("status_bar").show(ctx, |ui| {
            ui.label("Status bar");
            ui.set_min_width(150.0);
            ui.vertical(|ui| {
                let data = self.goods_to_show.0.last();
                match data {
                    Some(dailydata) => {
                        ui.label(format!("Event: {}", print_event(dailydata.event)));
                        ui.label(format!("Amount given: {}", dailydata.amount_given));
                        ui.label(format!("Amount received: {}", dailydata.amount_received));
                        ui.label(format!("Kind given: {}", print_kind(dailydata.kind_given)));
                        ui.label(format!(
                            "Kind received: {}",
                            print_kind(dailydata.kind_received)
                        ));
                    }
                    None => {
                        ui.label(format!("Event: "));
                        ui.label(format!("Amount given: "));
                        ui.label(format!("Amount received: "));
                        ui.label(format!("Kind given: "));
                        ui.label(format!("Kind received: "));
                    }
                }
                /* for (key, value) in self
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
                }*/
            });
        })
    }

    fn central_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Top down menu
            ui.menu_button("Choose Graph", |ui| {
                ui.selectable_value(&mut self.graph_choose, 0, "Plot");
                ui.selectable_value(&mut self.graph_choose, 1, "Bars");
            });
            ui.separator();

            let actual_index = self.get_actual_index();

            match self.graph_choose {
                0 => {
                    ui.label("Plot");
                    self.get_values();
                    let values = self.goods_to_show.1.clone();
                    println!(
                        "Actual index: {}, Index: {}, Max_Index: {}",
                        actual_index, self.index.1, self.index_max
                    );
                    egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        for curr in [Currency::EUR, Currency::USD, Currency::YEN, Currency::YUAN] {
                            match curr {
                                Currency::EUR => {
                                    let points = values[..actual_index]
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.eur])
                                        .collect_vec();
                                    ui.line(
                                        Line::new(PlotPoints::new(points)).name(print_kind(curr)),
                                    );
                                }
                                Currency::USD => {
                                    let points = values
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.usd])
                                        .collect_vec();
                                    ui.line(
                                        Line::new(PlotPoints::new(points)).name(print_kind(curr)),
                                    );
                                }
                                Currency::YEN => {
                                    let points = values
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.yen])
                                        .collect_vec();
                                    ui.line(
                                        Line::new(PlotPoints::new(points)).name(print_kind(curr)),
                                    );
                                }
                                Currency::YUAN => {
                                    let points = values
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.yuan])
                                        .collect_vec();
                                    ui.line(
                                        Line::new(PlotPoints::new(points)).name(print_kind(curr)),
                                    );
                                }
                            }
                        }
                    });
                }
                1 => {
                    ui.label("Bars");

                    let values = self.goods_to_show.1.clone();

                    let mut bars: Vec<Bar> = Vec::new();
                    for (i, curr) in
                        enumerate([Currency::EUR, Currency::USD, Currency::YEN, Currency::YUAN])
                    {
                        if values.len() == 0 {
                            break;
                        }
                        match curr {
                            Currency::EUR => {
                                let mut bar = egui::widgets::plot::Bar::new(
                                    i as f64,
                                    values[actual_index].eur,
                                );
                                bar = bar.name(print_kind(curr));
                                bar = bar.fill(egui::Color32::from_rgb(255, 0, 0));
                                bars.push(bar);
                            }
                            Currency::USD => {
                                let mut bar = egui::widgets::plot::Bar::new(
                                    i as f64,
                                    values[actual_index].usd,
                                );
                                bar = bar.name(print_kind(curr));
                                bar = bar.fill(egui::Color32::from_rgb(255, 0, 255));
                                bars.push(bar);
                            }
                            Currency::YEN => {
                                let mut bar = egui::widgets::plot::Bar::new(
                                    i as f64,
                                    values[actual_index].yen,
                                );
                                bar = bar.name(print_kind(curr));
                                bar = bar.fill(egui::Color32::from_rgb(255, 255, 0));
                                bars.push(bar);
                            }
                            Currency::YUAN => {
                                let mut bar = egui::widgets::plot::Bar::new(
                                    i as f64,
                                    values[actual_index].yuan,
                                );
                                bar = bar.name(print_kind(curr));
                                bar = bar.fill(egui::Color32::from_rgb(0, 255, 130));
                                bars.push(bar);
                            }
                        }
                    }

                    // Sort values
                    bars.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

                    let _plot = egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                        ui.bar_chart(BarChart::new(bars));
                    });
                }
                _ => {
                    ui.label("How did you get here?");
                    println!(
                        "
             ⣠⣤⣤⣤⣤⣤⣤⣤⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⡿⠛⠉⠙⠛⠛⠛⠛⠻⢿⣿⣷⣤⡀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⠋⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⠈⢻⣿⣿⡄⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣸⣿⡏⠀⠀⠀⣠⣶⣾⣿⣿⣿⠿⠿⠿⢿⣿⣿⣿⣄⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣿⣿⠁⠀⠀⢰⣿⣿⣯⠁⠀⠀⠀⠀⠀⠀⠀⠈⠙⢿⣷⡄⠀
⠀⠀⣀⣤⣴⣶⣶⣿⡟⠀⠀⠀⢸⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣷⠀
⠀⢰⣿⡟⠋⠉⣹⣿⡇⠀⠀⠀⠘⣿⣿⣿⣿⣷⣦⣤⣤⣤⣶⣶⣶⣶⣿⣿⣿⠀
⠀⢸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀
⠀⣸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠉⠻⠿⣿⣿⣿⣿⡿⠿⠿⠛⢻⣿⡇⠀⠀
⠀⣿⣿⠁⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣧⠀⠀
⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
⠀⢿⣿⡆⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀
⠀⠸⣿⣧⡀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠃⠀⠀
⠀⠀⠛⢿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⣰⣿⣿⣷⣶⣶⣶⣶⠶⢠⣿⣿⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⣽⣿⡏⠁⠀⠀⢸⣿⡇⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⢹⣿⡆⠀⠀⠀⣸⣿⠇⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢿⣿⣦⣄⣀⣠⣴⣿⣿⠁⠀⠈⠻⣿⣿⣿⣿⡿⠏⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠛⠻⠿⠿⠿⠿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                    "
                    );
                }
            }
        })
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(100));

        self.top_panel(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.central_panel(ctx);
    }
}
