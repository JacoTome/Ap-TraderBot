// Disable cmd window on windows release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use bose::market::BoseMarket;
use rcnz_market::rcnz::RCNZ;
use serde::Serialize;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber;
use trader::Trader;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;
use BVC::BVCMarket;

pub mod trader;
#[derive(Debug, Clone, Serialize)]
pub struct TraderState {
    name: String,
    goods: HashMap<GoodKind, f32>,
    earnings: f32,
    budget: f32,
}

unsafe impl Send for TraderState {}
unsafe impl Sync for TraderState {}
struct AsyncProcInputTx {
    inner: Mutex<mpsc::Sender<String>>,
}

pub struct AsyncProcInputTraderTx {
    inner: Mutex<mpsc::Sender<TraderState>>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<String>(1);

    let (async_proc_input_trader_tx, async_proc_input_trader_rx) = mpsc::channel::<TraderState>(2);
    let (async_proc_output_trader_tx, mut async_proc_output_trader_rx) =
        mpsc::channel::<TraderState>(2);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_input_tx),
        })
        .invoke_handler(tauri::generate_handler![js2rs])
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                trader::run(async_proc_input_rx, async_proc_output_trader_tx)
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // A loop that takes output from the async process and sends it
                // to the webview via a Tauri Event
                loop {
                    if let Some(output) = async_proc_output_trader_rx.recv().await {
                        rs2js(output, &app_handle);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// A function that sends a message from Rust to JavaScript via a Tauri Event
fn rs2js<R: tauri::Runtime>(message: TraderState, manager: &impl Manager<R>) {
    println!("{:?}", message);
    manager.emit_all("rs2js", message).unwrap();
}

// The Tauri command that gets called when Tauri `invoke` JavaScript API is
// called
#[tauri::command]
async fn js2rs(message: String, state: tauri::State<'_, AsyncProcInputTx>) -> Result<(), String> {
    info!(?message, "get_trader_state");
    let async_proc_input_tx = state.inner.lock().await;
    async_proc_input_tx
        .send(message)
        .await
        .map_err(|e| e.to_string())
}
