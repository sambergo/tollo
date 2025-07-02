mod m3u_parser;
mod database;
mod image_cache;
mod state;
mod channels;
mod settings;
mod playlists;
mod image_cache_api;
mod groups;
mod filters;
mod utils;

use std::sync::Mutex;
use tauri::Manager;
use image_cache::ImageCache;
use state::{DbState, ImageCacheState, ChannelCacheState};

// Import all the command functions from their respective modules
use channels::*;
use settings::*;
use playlists::*;
use image_cache_api::*;
use groups::*;
use filters::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut db_connection = database::initialize_database().expect("Failed to initialize database");
    
    // Run cleanup on startup to remove orphaned channel list files
    if let Err(e) = utils::cleanup_orphaned_channel_files(&db_connection) {
        println!("Warning: Channel list cleanup failed: {}", e);
    }
    
    let channels = m3u_parser::get_channels(&mut db_connection, None);
    database::populate_channels(&mut db_connection, &channels).expect("Failed to populate channels");

    tauri::Builder::default()
        .manage(DbState {
            db: Mutex::new(db_connection),
        })
        .manage(ChannelCacheState {
            cache: Mutex::new(None),
        })
        .setup(|app| {
            let image_cache = ImageCache::new(app.handle()).expect("Failed to initialize image cache");
            app.manage(ImageCacheState {
                cache: Mutex::new(image_cache),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Channel commands
            get_channels,
            get_groups,
            play_channel,
            add_favorite,
            remove_favorite,
            get_favorites,
            get_history,
            search_channels,
            invalidate_channel_cache,
            // Settings commands
            get_player_command,
            set_player_command,
            get_cache_duration,
            set_cache_duration,
            get_enable_preview,
            set_enable_preview,
            get_mute_on_start,
            set_mute_on_start,
            get_show_controls,
            set_show_controls,
            get_autoplay,
            set_autoplay,
            // Playlist commands
            get_channel_lists,
            add_channel_list,
            set_default_channel_list,
            refresh_channel_list,
            validate_and_add_channel_list,
            delete_channel_list,
            update_channel_list,
            // Image cache commands
            get_cached_image,
            clear_image_cache,
            get_image_cache_size,
            // Group commands
            get_enabled_groups,
            update_group_selection,
            sync_channel_list_groups,
            enable_all_groups,
            // Filter commands
            save_filter,
            get_saved_filters,
            delete_saved_filter,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
