mod m3u_parser;

use std::process::Command;
use m3u_parser::Channel;

#[tauri::command]
fn get_channels() -> Vec<Channel> {
    m3u_parser::get_channels()
}

#[tauri::command]
fn play_channel(url: String) {
    Command::new("mpv")
        .arg(url)
        .spawn()
        .expect("Failed to launch mpv");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_channels, play_channel])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}