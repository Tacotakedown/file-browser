// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_browser;
mod ssh;

use std::sync::Mutex;

use crate::file_browser::{connect, disconnect, download_file, list_files, upload_file, AppState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            ssh_client: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            list_files,
            download_file,
            upload_file,
            disconnect
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
