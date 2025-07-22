use tauri::State;
use crate::state::DbState;
use crate::database;

#[tauri::command]
pub fn get_enabled_groups(state: State<DbState>, channel_list_id: i64) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    database::get_enabled_groups(&db, channel_list_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_group_selection(state: State<DbState>, channel_list_id: i64, group_name: String, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::set_group_enabled(&db, channel_list_id, group_name, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_channel_list_groups(state: State<DbState>, channel_list_id: i64, groups: Vec<String>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    database::sync_channel_list_groups(&mut db, channel_list_id, groups).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn enable_all_groups(state: State<DbState>, channel_list_id: i64, groups: Vec<String>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    database::enable_all_groups(&mut db, channel_list_id, groups).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_all_groups(state: State<DbState>, channel_list_id: i64, groups: Vec<String>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    database::disable_all_groups(&mut db, channel_list_id, groups).map_err(|e| e.to_string())
} 