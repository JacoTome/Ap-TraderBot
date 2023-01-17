// Disable cmd window on windows release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Serialize;
use std::io::BufRead;
use std::io::BufReader;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing_subscriber;
struct AsyncProcInputTx {
    inner: Mutex<mpsc::Sender<String>>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel::<String>(1);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_input_tx),
        })
        .invoke_handler(tauri::generate_handler![js2rs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// A function that sends a message from Rust to JavaScript via a Tauri Event
// fn rs2js<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
//     println!("{:?}", message);
//     manager.emit_all("rs2js", message).unwrap();
// }

// The Tauri command that gets called when Tauri `invoke` JavaScript API is
// called
#[derive(Debug, Clone, Serialize)]
struct TraderState {
    name: String,
    earnings: f32,
    budget: f32,
}
#[tauri::command]
async fn js2rs(message: String, state: tauri::State<'_, AsyncProcInputTx>) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let mut output = std::process::Command::new(
            "C:\\Users\\jacot\\Documents\\trader-bot\\src-tauri\\src\\trader.exe",
        )
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");
        if let Some(stdout) = &mut output.stdout {
            let lines = BufReader::new(stdout);
            for line in lines.lines() {
                let line = line.unwrap();
                // Parse trader state
            }
        }
    });
    return Ok(());
}
