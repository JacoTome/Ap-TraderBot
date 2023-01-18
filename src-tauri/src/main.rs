// Disable cmd window on windows release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use lazy_static::lazy_static;
use serde::Serialize;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::Manager;
use tracing_subscriber;

use std::sync::Once;

fn get_handler() -> &'static Mutex<JoinHandle<()>> {
    static mut HANDLER: Option<Mutex<JoinHandle<()>>> = None;
    static HANDLER_ONCE: Once = Once::new();
    unsafe {
        HANDLER_ONCE
            .call_once(|| HANDLER = Some(Mutex::new(tauri::async_runtime::spawn(async move {}))));
        HANDLER.as_ref().unwrap()
    }
}

// static mut TRADER: Mutex<tauri::async_runtime::JoinHandle<()>> =
//     Mutex::new(tauri::async_runtime::spawn(async move || return Ok(())));
fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![js2rs, close_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// A function that sends a message from Rust to JavaScript via a Tauri Event
fn rs2js<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
    println!("sending: {:?}", message);
    manager.emit_all("rs2js", message).unwrap();
}

// The Tauri command that gets called when Tauri `invoke` JavaScript API is
// called
// #[derive(Debug, Clone, Serialize)]
// struct TraderState {
//     name: String,
//     earnings: f32,
//     budget: f32,
// }

#[tauri::command]
fn close_process() {
    println!("close_process called");
    let handler = get_handler().lock().unwrap();
    println!("handler: {:?}", handler);

    handler.abort();
}

#[tauri::command]
fn js2rs(window: tauri::Window) {
    print!("js2rs called");
    let mut command = std::process::Command::new(
        "C:\\Users\\jacot\\Documents\\trader-bot\\trader\\target\\debug\\trader.exe",
    );

    *get_handler().lock().unwrap() = tauri::async_runtime::spawn(async move {
        let output = command
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let child_stdout = output.stdout.unwrap();
        let lines = BufReader::new(child_stdout).lines();
        for line in lines.into_iter() {
            if let Ok(line) = line {
                window
                    .emit_all("rs2js", line)
                    .expect("failed to emit event");
            }
        }
    });
}
