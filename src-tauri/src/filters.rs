use tauri::State;
use crate::state::DbState;
use crate::database;

#[tauri::command]
pub fn save_filter(state: State<DbState>, channel_list_id: i64, slot_number: i32, search_query: String, selected_group: Option<String>, name: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::save_filter(&db, channel_list_id, slot_number, search_query, selected_group, name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_saved_filters(state: State<DbState>, channel_list_id: i64) -> Result<Vec<database::SavedFilter>, String> {
    let db = state.db.lock().unwrap();
    database::get_saved_filters(&db, channel_list_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_saved_filter(state: State<DbState>, channel_list_id: i64, slot_number: i32) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::delete_saved_filter(&db, channel_list_id, slot_number).map_err(|e| e.to_string())
} 