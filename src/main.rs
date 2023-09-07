pub mod data_models;
pub mod trader;
pub mod utils;

#[macro_use]
extern crate lazy_static;
use core::time::Duration;
use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, PlotPoints},
    style::{Selection, WidgetVisuals},
    Color32, RichText, Rounding, Stroke, Style, Ui, Vec2, Visuals,
};
use itertools::{enumerate, Itertools};
use std::{
    collections::vec_deque::VecDeque, collections::HashMap, sync::Mutex, thread, thread::JoinHandle,
};
use trader::trader_ricca;

use utils::utils::{print_event, print_kind};

use crate::{
    data_models::market::TraderTrait,
    trader::{main::Trader, trader_macca},
};
use data_models::market::{CurrencyData, DailyCurrencyData, DailyData, MarketData};
use unitn_market_2022::good::good_kind::GoodKind;
const STRATEGIES: &'static [&'static str] = &[
    "Default", "Prova1", "Prova2", "Prova3", "Prova4", "Prova5", "Prova6",
]; // TODO: INSERT STRATEGIES HERE
pub type TypeMarket = Vec<Vec<MarketData>>;

const EUR_COLOR: Color32 = Color32::from_rgb(203, 166, 247);
const USD_COLOR: Color32 = Color32::from_rgb(250, 179, 135);
const YEN_COLOR: Color32 = Color32::from_rgb(166, 227, 161);
const YUAN_COLOR: Color32 = Color32::from_rgb(137, 220, 235);

const BG_COLOR: Color32 = Color32::from_rgb(30, 30, 46);
const TEXT_COLOR: Color32 = Color32::from_rgb(205, 214, 244);
const DETAILS_COLOR: Color32 = Color32::from_rgb(17, 17, 27);
const PLOT_BG_COLOR: Color32 = Color32::from_rgb(24, 24, 37);
const INACTIVE_COLOR: Color32 = Color32::from_rgb(49, 50, 68);
const HOVERED_COLOR: Color32 = Color32::from_rgb(69, 71, 90);
const ACTIVE_COLOR: Color32 = Color32::from_rgb(88, 91, 112);
const PAUSED_COLOR: Color32 = Color32::from_rgb(243, 139, 168);
const RUNNING_COLOR: Color32 = Color32::from_rgb(166, 227, 161);

lazy_static! {
    pub static ref RUNNING_RICCA: Mutex<bool> = Mutex::new(false);
    pub static ref RUNNING_MACCA: Mutex<bool> = Mutex::new(false);
    pub static ref MARKETS_RICCA: Mutex<TypeMarket> = Mutex::new(Vec::new());
    pub static ref MARKETS_MACCA: Mutex<TypeMarket> = Mutex::new(Vec::new());
    pub static ref TRADER_DATA_RICCA: Mutex<VecDeque<DailyCurrencyData>> =
        Mutex::new(VecDeque::new());
    pub static ref TRADER_DATA_MACCA: Mutex<VecDeque<DailyCurrencyData>> =
        Mutex::new(VecDeque::new());
    pub static ref SELECTED_STRATEGY: Mutex<String> = Mutex::new(STRATEGIES[0].to_string());
}

const TRADERS_NAME_RICCA: &str = "Ricca";
const TRADERS_NAME_MACCA: &str = "Maccacaro";
fn is_running(key: String) -> bool {
    match key.as_str() {
        TRADERS_NAME_RICCA => match RUNNING_RICCA.lock() {
            Ok(binding) => *binding,
            Err(e) => {
                println!("Error: {}", e);
                false
            }
        },
        TRADERS_NAME_MACCA => match RUNNING_MACCA.lock() {
            Ok(binding) => *binding,
            Err(e) => {
                println!("Error: {}", e);
                false
            }
        },
        _ => {
            println!("Error: Trader not found");
            false
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        always_on_top: false,
        initial_window_size: Some(egui::Vec2::new(1200.0, 800.0)),
        resizable: true,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "Trader visualization",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}

const STRAT: i32 = 3;

struct Data {
    goods_to_show: (Vec<DailyData>, Vec<CurrencyData>),
    history_daily_data: VecDeque<DailyData>,
    history_currencies: VecDeque<CurrencyData>,
    markets_data: HashMap<String, Vec<CurrencyData>>,
    index: (bool, usize), // 0 = unchanged, 1 = changed
    index_max: usize,
}
struct MyApp {
    curr_to_show: HashMap<GoodKind, bool>,
    graph_choose: usize,
    plot_choose: usize,
    load_value: usize,
    traders: HashMap<String, Option<JoinHandle<()>>>,
    traders_data: HashMap<String, Data>,
    selected_trader: String, // market_goods: HashMap<String, Vec<GoodLabel>>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let graph_choose = 0;
        let plot_choose: usize = 0;

        let mut curr_to_show = HashMap::new();
        for curr in [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
            curr_to_show.insert(curr, true);
        }
        let mut traders = HashMap::new();
        let mut traders_data = HashMap::new();
        traders_data.insert(
            TRADERS_NAME_RICCA.to_string(),
            Data {
                goods_to_show: (vec![], vec![]),
                history_daily_data: VecDeque::new(),
                history_currencies: VecDeque::new(),
                markets_data: HashMap::new(),
                index: (true, 0),
                index_max: 0,
            },
        );
        traders_data.insert(
            TRADERS_NAME_MACCA.to_string(),
            Data {
                goods_to_show: (vec![], vec![]),
                history_daily_data: VecDeque::new(),
                history_currencies: VecDeque::new(),
                markets_data: HashMap::new(),
                index: (true, 0),
                index_max: 0,
            },
        );
        traders.insert("Ricca".to_string(), None);
        traders.insert("Maccacaro".to_string(), None);

        Self {
            curr_to_show: curr_to_show,
            plot_choose,
            graph_choose,
            load_value: 0,
            traders: traders,
            traders_data: traders_data,
            selected_trader: TRADERS_NAME_RICCA.to_string(),
        }
    }

    fn get_actual_index(&self) -> usize {
        let key = self.selected_trader.to_string();
        let data = self.traders_data.get(&key).unwrap();
        if data.index.0 {
            if data.goods_to_show.1.len() == 0 {
                return 0;
            }
            data.goods_to_show.1.len() - 1
        } else {
            data.index.1 - 1
        }
    }

    fn minus_one_day(&mut self, key: String) {
        let data = self.traders_data.get_mut(&key).unwrap();
        if data.index.1 > 0 {
            data.index.0 = false;
            data.index.1 -= 1;
        }
    }

    fn plus_one_day(&mut self, key: String) {
        let data = self.traders_data.get_mut(&key).unwrap();

        if data.index.1 < data.index_max {
            data.index.0 = false;
            data.index.1 += 1;
        }
    }

    fn go_to_day(&mut self, day: usize) {
        let data = self
            .traders_data
            .get_mut(&self.selected_trader.clone())
            .unwrap();

        if day < data.index_max {
            data.index.0 = false;
            data.index.1 = day;
        }
    }

    fn get_values(&mut self) {
        let data = self
            .traders_data
            .get_mut(self.selected_trader.as_str())
            .unwrap();
        if !data.history_currencies.is_empty() {
            data.goods_to_show
                .0
                .push(data.history_daily_data.pop_front().unwrap());
            data.goods_to_show
                .1
                .push(data.history_currencies.pop_front().unwrap());
            data.index_max = data.goods_to_show.1.len().max(data.goods_to_show.0.len());
            if data.index.0 {
                data.index.1 = data.index_max;
            }
        } else {
            match self.selected_trader.as_str() {
                "Ricca" => self.update_data_ricca(),
                "Maccacaro" => self.update_data_macca(),
                _ => {}
            }
        }
    }

    fn update_data_macca(&mut self) {
        match TRADER_DATA_MACCA.lock() {
            Ok(mut data) => {
                while !data.is_empty() {
                    let new_data = data.pop_front().unwrap().clone();
                    let trader_data = self.traders_data.get_mut("Maccacaro").unwrap();
                    trader_data
                        .history_currencies
                        .push_back(new_data.currencies.clone());
                    trader_data
                        .history_daily_data
                        .push_back(new_data.daily_data.clone());
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        }

        match MARKETS_MACCA.lock() {
            Ok(mut data) => {
                while !data.is_empty() {
                    let new_data = data.pop().unwrap().clone();
                    let trader_data = self.traders_data.get_mut("Maccacaro").unwrap();
                    for data in new_data {
                        let binding = trader_data
                            .markets_data
                            .entry(data.name)
                            .or_insert(Vec::new());
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

    fn update_data_ricca(&mut self) {
        match TRADER_DATA_RICCA.lock() {
            Ok(mut data) => {
                while !data.is_empty() {
                    let new_data = data.pop_front().unwrap().clone();
                    let trader_data = self.traders_data.get_mut("Ricca").unwrap();

                    trader_data
                        .history_currencies
                        .push_back(new_data.currencies.clone());
                    trader_data
                        .history_daily_data
                        .push_back(new_data.daily_data.clone());
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        }

        match MARKETS_RICCA.lock() {
            Ok(mut data) => {
                while !data.is_empty() {
                    let new_data = data.pop().unwrap().clone();
                    let trader_data = self.traders_data.get_mut("Ricca").unwrap();
                    for data in new_data {
                        let binding = trader_data
                            .markets_data
                            .entry(data.name)
                            .or_insert(Vec::new());
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
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Macca").clicked() {
                        self.selected_trader = TRADERS_NAME_MACCA.to_string();
                    }
                    if ui.button("Ricca").clicked() {
                        self.selected_trader = TRADERS_NAME_RICCA.to_string();
                    }
                });
                ui.horizontal(|ui| {
                    ui.set_height(50.0);
                    ui.label(
                        egui::RichText::new("Trader visualization")
                            .size(15.0)
                            .color(Color32::YELLOW),
                    );

                    let load_all_button = ui.button("-1 Day").on_hover_text("Load all values");
                    if load_all_button.clicked() {
                        match self.selected_trader.as_str() {
                            "Ricca" => {
                                self.minus_one_day("Ricca".to_string());
                            }
                            "Maccacaro" => {
                                self.minus_one_day("Maccacaro".to_string());
                            }
                            _ => {}
                        }
                    }

                    let load_one_button = ui.button("+1 Day").on_hover_text("Load one value");
                    if load_one_button.clicked() {
                        match self.selected_trader.as_str() {
                            "Ricca" => {
                                self.plus_one_day("Ricca".to_string());
                            }
                            "Maccacaro" => {
                                self.plus_one_day("Maccacaro".to_string());
                            }
                            _ => {}
                        }
                    }

                    ui.label("Select day");
                    let mut val_string = format!("").to_string();
                    let value = ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut val_string).hint_text(String::from("Day")));

                    let load_value = ui.button("Load");
                    if value.changed() {
                        if val_string.parse::<usize>().is_ok() {
                            self.load_value = val_string.parse::<usize>().unwrap();
                        }
                    }

                    if load_value.clicked() || ui.input().key_pressed(egui::Key::Enter) {
                        self.go_to_day(self.load_value);
                    }

                    ui.label(format!(
                        "Day: {}",
                        self.traders_data
                            .get(self.selected_trader.as_str())
                            .unwrap()
                            .index
                            .1
                    ));

                    let start_button = ui.button("Start/Stop");
                    if start_button.clicked() {
                        match self.selected_trader.as_str() {
                            "Ricca" => {
                                let mut binding = RUNNING_RICCA.lock().unwrap();
                                *binding = !*binding;
                                self.traders_data.get_mut("Ricca").unwrap().index.0 = true;
                            }
                            "Maccacaro" => {
                                let mut binding = RUNNING_MACCA.lock().unwrap();
                                *binding = !*binding;
                                self.traders_data.get_mut("Maccacaro").unwrap().index.0 = true;
                            }
                            _ => {}
                        }

                        for (key, mut value) in self.traders.iter_mut() {
                            let key_to_pass = key.clone();
                            match value {
                                Some(thread) => {
                                    thread.thread().unpark();
                                }
                                None => {
                                    value = &mut Some(thread::spawn(move || {
                                        match key_to_pass.as_str() {
                                            "Ricca" => {
                                                /********************
                                                       TRADER
                                                ********************/
                                                let mut trader = Trader::new(
                                                &RUNNING_RICCA,
                                                &MARKETS_RICCA,
                                                &TRADER_DATA_RICCA,
                                                &SELECTED_STRATEGY,
                                                Box::new(
                                                    trader_ricca::trader_ricca::initialize_trader(
                                                        STRAT,
                                                    ),
                                                ),
                                            );
                                                println!("Trader started");
                                                loop {
                                                    if trader.is_running() {
                                                        trader.pass_one_day();
                                                        thread::sleep(
                                                            std::time::Duration::from_millis(100),
                                                        );
                                                    } else {
                                                        println!("Parked");
                                                        thread::park();
                                                    }
                                                }
                                            }
                                            "Maccacaro" => {
                                                /********************
                                                       TRADER
                                                ********************/
                                                let mut trader = Trader::new(
                                                &RUNNING_MACCA,
                                                &MARKETS_MACCA,
                                                &TRADER_DATA_MACCA,
                                                &SELECTED_STRATEGY,
                                                Box::new(
                                                    trader_macca::trader_maccacaro::initialize_trader(
                                                        STRAT,
                                                    ),
                                                ),
                                            );
                                                println!("Trader started");
                                                loop {
                                                    if trader.is_running() {
                                                        trader.pass_one_day();
                                                        thread::sleep(
                                                            std::time::Duration::from_millis(100),
                                                        );
                                                    } else {
                                                        println!("Parked");
                                                        thread::park();
                                                    }
                                                }
                                            }
                                            _ => {
                                                println!("Error: Trader not found");
                                            }
                                        }
                                    }));
                                }
                            }
                        }
                    }

                    if is_running(self.selected_trader.to_string()) {
                        ui.colored_label(RUNNING_COLOR, format!("{} is running", self.selected_trader));
                    } else {
                        ui.colored_label(PAUSED_COLOR, format!("{} is stopped", self.selected_trader));
                    }
                });
            });
        })
    }

    fn left_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label(RichText::new("Select goods to show").heading());
            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal_top(|ui| {
                    let select_all_button = ui.button("Select all");
                    if select_all_button.clicked() {
                        for (_key, value) in self.curr_to_show.iter_mut() {
                            *value = true;
                        }
                        /* for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = true;
                        }*/
                    }
                    let deselect_all_button = ui.button("Deselect all");
                    if deselect_all_button.clicked() {
                        for (_key, value) in self.curr_to_show.iter_mut() {
                            *value = false;
                        }
                        /*    for (_key, value) in self.goods_to_show.iter_mut() {
                            value.0 = false;
                        }*/
                    }
                });
                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    for (key, value) in self.curr_to_show.iter_mut() {
                        ui.checkbox(
                            value,
                            egui::RichText::new(print_kind(*key))
                                .color(match key {
                                    GoodKind::EUR => EUR_COLOR,
                                    GoodKind::USD => USD_COLOR,
                                    GoodKind::YEN => YEN_COLOR,
                                    GoodKind::YUAN => YUAN_COLOR,
                                })
                                .size(18.0),
                        );
                    }
                });
            });
        })
    }

    fn right_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        self.get_values();

        egui::SidePanel::right("status_bar").show(ctx, |ui| {
            ui.label(RichText::new("Status bar").heading());
            ui.set_min_width(150.0);
            ui.vertical(|ui| {
                let data = self
                    .traders_data
                    .get(self.selected_trader.as_str())
                    .unwrap()
                    .goods_to_show
                    .0
                    .last();
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
        ui.vertical_centered(|ui| {
            let actual_index = self.get_actual_index();
            ui.menu_button("Choose Graph", |ui| {
                ui.selectable_value(&mut self.graph_choose, 0, "Plot");
                ui.selectable_value(&mut self.graph_choose, 1, "Bars");
            });
            ui.add_space(20.0);

            match self.graph_choose {
                0 => {
                    let values = self
                        .traders_data
                        .get(self.selected_trader.as_str())
                        .unwrap()
                        .goods_to_show
                        .1
                        .clone();
                    egui::widgets::plot::Plot::new("my_plot")
                        .width(800.0)
                        .view_aspect(16.0 / 9.0)
                        .show(ui, |ui| {
                            for curr in
                                [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN]
                            {
                                match curr {
                                    GoodKind::EUR => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = values[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.eur])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name(print_kind(curr))
                                                    .color(EUR_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::USD => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = values[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.usd])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name(print_kind(curr))
                                                    .color(USD_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::YEN => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = values[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.yen])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name(print_kind(curr))
                                                    .color(YEN_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::YUAN => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = values[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.yuan])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name(print_kind(curr))
                                                    .color(YUAN_COLOR),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                }
                1 => {
                    let values = self
                        .traders_data
                        .get(self.selected_trader.as_str())
                        .unwrap()
                        .goods_to_show
                        .1
                        .clone();

                    let mut bars: Vec<Bar> = Vec::new();
                    for (i, curr) in
                        enumerate([GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN])
                    {
                        if values.len() == 0 {
                            break;
                        }
                        match curr {
                            GoodKind::EUR => {
                                if *self.curr_to_show.get(&curr).unwrap() {
                                    let mut bar = egui::widgets::plot::Bar::new(
                                        i as f64,
                                        values[actual_index].eur,
                                    );
                                    bar = bar.name(print_kind(curr));
                                    bar = bar.fill(EUR_COLOR);
                                    bars.push(bar);
                                }
                            }
                            GoodKind::USD => {
                                if *self.curr_to_show.get(&curr).unwrap() {
                                    let mut bar = egui::widgets::plot::Bar::new(
                                        i as f64,
                                        values[actual_index].usd,
                                    );
                                    bar = bar.name(print_kind(curr));
                                    bar = bar.fill(USD_COLOR);
                                    bars.push(bar);
                                }
                            }
                            GoodKind::YEN => {
                                if *self.curr_to_show.get(&curr).unwrap() {
                                    let mut bar = egui::widgets::plot::Bar::new(
                                        i as f64,
                                        values[actual_index].yen,
                                    );
                                    bar = bar.name(print_kind(curr));
                                    bar = bar.fill(YEN_COLOR);
                                    bars.push(bar);
                                }
                            }
                            GoodKind::YUAN => {
                                if *self.curr_to_show.get(&curr).unwrap() {
                                    let mut bar = egui::widgets::plot::Bar::new(
                                        i as f64,
                                        values[actual_index].yuan,
                                    );
                                    bar = bar.name(print_kind(curr));
                                    bar = bar.fill(YUAN_COLOR);
                                    bars.push(bar);
                                }
                            }
                        }
                    }

                    // Sort values
                    bars.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

                    egui::widgets::plot::Plot::new("my_plot")
                        .width(800.0)
                        .view_aspect(16.0 / 9.0)
                        .show(ui, |ui| {
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
        });
    }

    fn show_market(&mut self, ui: &mut Ui) {
        let actual_index = self.get_actual_index();
        let values = self
            .traders_data
            .get(self.selected_trader.as_str())
            .unwrap()
            .markets_data
            .clone();
        let number_markets = values.len();
        ui.horizontal_wrapped(|ui| {
            ui.set_max_width(800.0);
            for (key, value) in values {
                ui.vertical(|ui| {
                    ui.label("Market: ".to_string() + &key);
                    egui::widgets::plot::Plot::new(key.clone())
                        .height(800.0 / number_markets as f32)
                        .view_aspect(1.0)
                        .show(ui, |ui| {
                            for curr in
                                [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN]
                            {
                                match curr {
                                    GoodKind::EUR => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = value[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.eur])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name("Eur")
                                                    .color(EUR_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::USD => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = value[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.usd])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name("Usd")
                                                    .color(USD_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::YEN => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = value[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.yen])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name("Yen")
                                                    .color(YEN_COLOR),
                                            );
                                        }
                                    }
                                    GoodKind::YUAN => {
                                        if *self.curr_to_show.get(&curr).unwrap() {
                                            let points = value[..actual_index]
                                                .iter()
                                                .enumerate()
                                                .map(|(i, &v)| [i as f64, v.yuan])
                                                .collect_vec();
                                            ui.line(
                                                Line::new(PlotPoints::new(points))
                                                    .name("Yuan")
                                                    .color(YUAN_COLOR),
                                            );
                                        }
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
                ui.set_style(Style {
                    spacing: egui::style::Spacing {
                        item_spacing: Vec2 { x: 5.0, y: 5.0 },
                        ..Default::default()
                    },
                    ..Default::default()
                });
                ui.set_min_size(Vec2::new(200.0, 10.0));
                ui.set_max_height(20.0);
                ui.horizontal_top(|ui| {
                    if ui.button("Show Markets").clicked() {
                        self.plot_choose = 1;
                    }
                    ui.add_space(10.0);
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

    fn bottom_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::bottom("Statistics").show(ctx, |ui| {
            ui.set_min_height(100.0);
            ui.label("Statistics");
            let values = self
                .traders_data
                .get(self.selected_trader.as_str())
                .unwrap()
                .goods_to_show
                .1
                .clone();
            let last_currency = values.last();
            if last_currency.is_none() {
                return;
            }
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Last currency: ");
                    ui.label(format!("EUR: {}", last_currency.unwrap().eur));
                    ui.label(format!("USD: {}", last_currency.unwrap().usd));
                    ui.label(format!("YEN: {}", last_currency.unwrap().yen));
                    ui.label(format!("YUAN: {}", last_currency.unwrap().yuan));
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Difference in 50 days: ");
                    if values.len() > 50 {
                        ui.label(format!(
                            "EUR: {}",
                            last_currency.unwrap().eur - values[values.len() - 50].eur
                        ));
                        ui.label(format!(
                            "USD: {}",
                            last_currency.unwrap().usd - values[values.len() - 50].usd
                        ));
                        ui.label(format!(
                            "YEN: {}",
                            last_currency.unwrap().yen - values[values.len() - 50].yen
                        ));
                        ui.label(format!(
                            "YUAN: {}",
                            last_currency.unwrap().yuan - values[values.len() - 50].yuan
                        ));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Total gain: ");
                    ui.label(format!(
                        "EUR: {}",
                        last_currency.unwrap().eur - values[0].eur
                    ));
                    ui.label(format!(
                        "USD: {}",
                        last_currency.unwrap().usd - values[0].usd
                    ));
                    ui.label(format!(
                        "YEN: {}",
                        last_currency.unwrap().yen - values[0].yen
                    ));
                    ui.label(format!(
                        "YUAN: {}",
                        last_currency.unwrap().yuan - values[0].yuan
                    ));
                })
            });
        })
    }
}

impl<'a> eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(100));

        let non_interactive_style = WidgetVisuals {
            fg_stroke: Stroke::new(2.0, DETAILS_COLOR),
            bg_stroke: Stroke::new(2.0, DETAILS_COLOR),
            bg_fill: BG_COLOR,
            rounding: Rounding::none(),
            expansion: 10.0,
        };

        let inactive = WidgetVisuals {
            fg_stroke: Stroke::new(1.0, TEXT_COLOR),
            bg_stroke: Stroke::new(1.0, INACTIVE_COLOR),
            bg_fill: INACTIVE_COLOR,
            rounding: Rounding {
                nw: 5.0,
                ne: 5.0,
                se: 5.0,
                sw: 5.0,
            },
            expansion: 1.0,
        };

        let hovered = WidgetVisuals {
            fg_stroke: Stroke::new(1.0, TEXT_COLOR),
            bg_stroke: Stroke::new(1.0, HOVERED_COLOR),
            bg_fill: HOVERED_COLOR,
            rounding: Rounding {
                nw: 5.0,
                ne: 5.0,
                se: 5.0,
                sw: 5.0,
            },
            expansion: 1.5,
        };

        let active = WidgetVisuals {
            fg_stroke: Stroke::new(1.0, TEXT_COLOR),
            bg_stroke: Stroke::new(1.0, ACTIVE_COLOR),
            bg_fill: ACTIVE_COLOR,
            rounding: Rounding {
                nw: 5.0,
                ne: 5.0,
                se: 5.0,
                sw: 5.0,
            },
            expansion: 1.5,
        };

        let open = WidgetVisuals {
            fg_stroke: Stroke::new(1.0, TEXT_COLOR),
            bg_stroke: Stroke::new(1.0, TEXT_COLOR),
            bg_fill: ACTIVE_COLOR,
            rounding: Rounding {
                nw: 5.0,
                ne: 5.0,
                se: 5.0,
                sw: 5.0,
            },
            expansion: 1.5,
        };

        let widgets = egui::style::Widgets {
            noninteractive: non_interactive_style,
            inactive: inactive,
            hovered: hovered,
            active: active,
            open: open,
        };

        let selection = Selection {
            bg_fill: HOVERED_COLOR,
            stroke: Stroke::new(1.0, TEXT_COLOR),
        };

        let visuals = Visuals {
            widgets: widgets,
            override_text_color: Some(TEXT_COLOR),
            window_fill: BG_COLOR,
            panel_fill: BG_COLOR,
            selection: selection,
            extreme_bg_color: PLOT_BG_COLOR,
            ..Default::default()
        };

        ctx.set_style(Style {
            visuals: visuals,
            ..Default::default()
        });

        self.top_panel(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.central_panel(ctx);
        self.bottom_panel(ctx);
    }
}
