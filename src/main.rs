pub mod data_models;
pub mod trader;
pub mod utils;

#[macro_use]
extern crate lazy_static;
use core::time::Duration;
use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, PlotPoints},
    Color32, InnerResponse, Ui, Vec2,
};
use itertools::{enumerate, Itertools};
use std::{
    collections::vec_deque::VecDeque, collections::HashMap, sync::Mutex, thread, thread::JoinHandle,
};
use trader::trader_ricca;

use utils::utils::{print_event, print_kind};

use crate::{data_models::market::TraderTrait, trader::main::Trader};
use data_models::market::{CurrencyData, DailyCurrencyData, DailyData, MarketData};
const STRATEGIES: &'static [&'static str] = &[
    "Default", "Prova1", "Prova2", "Prova3", "Prova4", "Prova5", "Prova6",
]; // TODO: INSERT STRATEGIES HERE
use unitn_market_2022::good::good_kind::GoodKind;
pub type TypeMarket = Vec<Vec<MarketData>>;

lazy_static! {
    pub static ref RUNNING: Mutex<bool> = Mutex::new(false);
    pub static ref MARKETS: Mutex<TypeMarket> = Mutex::new(Vec::new());
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

const STRAT: i32 = 3;
struct MyApp {
    goods_to_show: (Vec<DailyData>, Vec<CurrencyData>),
    graph_choose: usize,
    plot_choose: usize,
    load_value: usize,
    traders: (Option<JoinHandle<()>>, Option<JoinHandle<()>>),
    history_daily_data: VecDeque<DailyData>,
    history_currencies: VecDeque<CurrencyData>,
    markets_data: HashMap<String, Vec<CurrencyData>>,
    index: (bool, usize), // 0 = unchanged, 1 = changed
    index_max: usize,
    // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let graph_choose = 0;
        let plot_choose: usize = 0;

        let goods_to_show = (vec![], vec![]);
        //let _trader_maccaccaro;
        Self {
            goods_to_show,
            plot_choose,
            graph_choose,
            load_value: 0,
            traders: (None, None),
            history_daily_data: VecDeque::new(),
            history_currencies: VecDeque::new(),
            markets_data: HashMap::new(),
            index: (true, 0),
            index_max: 0,
        }
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
    }

    fn update_data(&mut self) {
        match TRADER_DATA.lock() {
            Ok(mut data) => {
                if data.is_empty() {
                    return;
                }
                while !data.is_empty() {
                    let new_data = data.pop_front().unwrap().clone();
                    self.history_currencies
                        .push_back(new_data.currencies.clone());
                    self.history_daily_data
                        .push_back(new_data.daily_data.clone());
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        }

        match MARKETS.lock() {
            Ok(mut data) => {
                if data.is_empty() {
                    return;
                }
                while !data.is_empty() {
                    let new_data = data.pop().unwrap().clone();
                    for data in new_data {
                        let mut binding = self.markets_data.entry(data.name).or_insert(Vec::new());
                        binding.push(data.currencies);
                    }
                }
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
                    todo!();
                }

                let start_button = ui.button("Start/Stop");
                if start_button.clicked() {
                    switch_run_pause();
                    self.index.0 = true;

                    // Multi trader
                    match &self.traders.0 {
                        Some(thread) => {
                            thread.thread().unpark();
                        }
                        None => {
                            self.traders.0 = Some(thread::spawn(move || {
                                /********************
                                       TRADER
                                ********************/
                                let mut trader = Trader::new(
                                    &RUNNING,
                                    &MARKETS,
                                    &TRADER_DATA,
                                    &SELECTED_STRATEGY,
                                    Box::new(trader_ricca::trader_ricca::initialize_trader(STRAT)),
                                );
                                println!("Trader started");
                                loop {
                                    if trader.is_running() {
                                        trader.pass_one_day();
                                        thread::sleep(std::time::Duration::from_millis(100));
                                    } else {
                                        println!("Parked");
                                        thread::park();
                                    }
                                }
                            }));
                        }
                    }
                }

                //     if is_running() {
                //         if self.trader_thread.is_none() {
                //             self.trader_thread =
                //                 Some(thread::spawn(move || {
                //                     /********************
                //                            TRADER
                //                     ********************/
                //                     let paused = Mutex::new(false);
                //                     let mut trader = Trader::new(
                //                     &RUNNING,
                //                     &MARKETS,
                //                     &TRADER_DATA,
                //                     &SELECTED_STRATEGY,
                //                     crate::trader::trader_ricca::trader_struct::initialize_trader(0)
                //                 );
                //                     println!("Trader started");
                //                     loop {
                //                         if is_running() {
                //                             trader.pass_one_day();
                //                             thread::sleep(std::time::Duration::from_millis(100));
                //                         } else {
                //                             println!("Parked");
                //                             thread::park();
                //                         }
                //                     }
                //                 }));
                //         } else {
                //             self.trader_thread.as_ref().unwrap().thread().unpark();
                //         }
                //     }
                // }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let reset_button = ui.button("Reset");
                    if reset_button.clicked() {
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
            });
        })
    }

    fn show_trader(&mut self, ui: &mut Ui) {
        let actual_index = self.get_actual_index();
        match self.graph_choose {
            0 => {
                ui.label("Plot");
                let values = self.goods_to_show.1.clone();
                egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                    for curr in [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
                        match curr {
                            GoodKind::EUR => {
                                let points = values[..actual_index]
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &v)| [i as f64, v.eur])
                                    .collect_vec();
                                ui.line(Line::new(PlotPoints::new(points)).name(print_kind(curr)));
                            }
                            GoodKind::USD => {
                                let points = values[..actual_index]
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &v)| [i as f64, v.usd])
                                    .collect_vec();
                                ui.line(Line::new(PlotPoints::new(points)).name(print_kind(curr)));
                            }
                            GoodKind::YEN => {
                                let points = values[..actual_index]
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &v)| [i as f64, v.yen])
                                    .collect_vec();
                                ui.line(Line::new(PlotPoints::new(points)).name(print_kind(curr)));
                            }
                            GoodKind::YUAN => {
                                let points = values[..actual_index]
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &v)| [i as f64, v.yuan])
                                    .collect_vec();
                                ui.line(Line::new(PlotPoints::new(points)).name(print_kind(curr)));
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
                    enumerate([GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN])
                {
                    if values.len() == 0 {
                        break;
                    }
                    match curr {
                        GoodKind::EUR => {
                            let mut bar =
                                egui::widgets::plot::Bar::new(i as f64, values[actual_index].eur);
                            bar = bar.name(print_kind(curr));
                            bar = bar.fill(egui::Color32::from_rgb(255, 0, 0));
                            bars.push(bar);
                        }
                        GoodKind::USD => {
                            let mut bar =
                                egui::widgets::plot::Bar::new(i as f64, values[actual_index].usd);
                            bar = bar.name(print_kind(curr));
                            bar = bar.fill(egui::Color32::from_rgb(255, 0, 255));
                            bars.push(bar);
                        }
                        GoodKind::YEN => {
                            let mut bar =
                                egui::widgets::plot::Bar::new(i as f64, values[actual_index].yen);
                            bar = bar.name(print_kind(curr));
                            bar = bar.fill(egui::Color32::from_rgb(255, 255, 0));
                            bars.push(bar);
                        }
                        GoodKind::YUAN => {
                            let mut bar =
                                egui::widgets::plot::Bar::new(i as f64, values[actual_index].yuan);
                            bar = bar.name(print_kind(curr));
                            bar = bar.fill(egui::Color32::from_rgb(0, 255, 130));
                            bars.push(bar);
                        }
                    }
                }

                // Sort values
                bars.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

                egui::widgets::plot::Plot::new("my_plot").show(ui, |ui| {
                    ui.bar_chart(BarChart::new(bars));
                });
            }
            _ => {
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
                ui.group(|ui| {
                    ui.label("Io non esisto");
                });
            }
        }
    }

    fn show_market(&mut self, ui: &mut Ui) {
        let actual_index = self.get_actual_index();
        let values = self.markets_data.clone();
        ui.horizontal_centered(|ui| {
            ui.label("Plot");
            ui.set_max_size(Vec2 {
                x: 1000.0,
                y: 500.0,
            });
            for (key, value) in values {
                ui.vertical(|ui| {
                    ui.set_max_size(Vec2 { x: 500.0, y: 500.0 });
                    ui.label("Market: ".to_string() + &key);
                    egui::widgets::plot::Plot::new(key.clone()).show(ui, |ui| {
                        for curr in [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
                            match curr {
                                GoodKind::EUR => {
                                    let points = value[..actual_index]
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.eur])
                                        .collect_vec();
                                    ui.line(Line::new(PlotPoints::new(points)).name("Eur"));
                                }
                                GoodKind::USD => {
                                    let points = value[..actual_index]
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.usd])
                                        .collect_vec();
                                    ui.line(Line::new(PlotPoints::new(points)).name("Usd"));
                                }
                                GoodKind::YEN => {
                                    let points = value[..actual_index]
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.yen])
                                        .collect_vec();
                                    ui.line(Line::new(PlotPoints::new(points)).name("Yen"));
                                }
                                GoodKind::YUAN => {
                                    let points = value[..actual_index]
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &v)| [i as f64, v.yuan])
                                        .collect_vec();
                                    ui.line(Line::new(PlotPoints::new(points)).name("Yuan"));
                                }
                            }
                        }
                    });
                });
            }
        });
    }

    fn central_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Top down menu
            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 10.0));
                ui.set_max_height(20.0);
                ui.horizontal_top(|ui| {
                    ui.menu_button("Choose Graph", |ui| {
                        ui.selectable_value(&mut self.graph_choose, 0, "Plot");
                        ui.selectable_value(&mut self.graph_choose, 1, "Bars");
                    });

                    if ui.button("Show Markets").clicked() {
                        self.plot_choose = 1;
                    }

                    if ui.button("Show Trader").clicked() {
                        self.plot_choose = 0;
                    }
                })
            });

            match self.plot_choose {
                0 => self.show_trader(ui),
                1 => self.show_market(ui),
                _ => {}
            }
        })
    }
}

impl<'a> eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(100));

        self.top_panel(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.central_panel(ctx);
    }
}
