// Disable cmd window on windows release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::utils::io;
use tauri::Manager;
use tracing::info;
use tracing::warn;
use tracing_subscriber;

use std::sync::Once;
// mod taskHandler;

lazy_static! {
    static ref MY_CHANNEL: Arc<Mutex<(Sender<String>, Receiver<String>)>> =
        Arc::new(Mutex::new(std::sync::mpsc::channel::<String>()));
}

fn get_handler() -> &'static Arc<Mutex<JoinHandle<()>>> {
    static mut HANDLER: Option<Arc<Mutex<JoinHandle<()>>>> = None;
    static HANDLER_ONCE: Once = Once::new();
    unsafe {
        HANDLER_ONCE.call_once(|| {
            HANDLER = Some(Arc::new(Mutex::new(tauri::async_runtime::spawn(
                async move {},
            ))))
        });
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
    info!("Closing process...");
    // Check if mutex is available

    // if let Ok(mut mutex) = get_handler().try_lock() {
    //     info!("Mutex is available");
    //     let handler = mutex.inner();
    //     println!("Handler: {:?}", handler);
    //     handler.abort();
    //     info!("Handler aborted")
    // } else {
    //     warn!("Mutex is not available")
    // }

    if let Ok(mutex) = MY_CHANNEL.try_lock() {
        match mutex.0.send("close".to_string()) {
            Ok(_) => {
                info!("Sent close message");
            }
            Err(e) => {
                info!("Error sending close message: {:?}", e);
            }
        }
    } else {
        warn!("Mutex is not available")
    }
}

#[tauri::command]
fn js2rs(window: tauri::Window) {
    println!("Send called");
    let mut command = std::process::Command::new(
        "C:\\Users\\jacot\\Documents\\trader-bot\\trader\\target\\debug\\trader.exe",
    );

    info!("Starting process trader");
    tauri::async_runtime::spawn(async move {
        let output = command
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        // let child_stdout = &mut
        let lines = BufReader::new(output.stdout.unwrap()).lines();
        for line in lines.into_iter() {
            if let Ok(line) = line {
                window
                    .emit_all("rs2js", &line)
                    .expect("failed to emit event");
                if let Ok(msg) = MY_CHANNEL.lock().unwrap().1.try_recv() {
                    if msg == "close" {
                        info!("Closing process");
                        command.stdout(std::process::Stdio::null());
                        return;
                    }
                }
            }
        }
    });

    info!("Done");
}
