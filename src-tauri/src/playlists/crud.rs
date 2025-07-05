use crate::channels::invalidate_channel_cache;
use crate::state::{ChannelCacheState, ChannelList, DbState};
use tauri::State;

#[tauri::command]
pub fn get_channel_lists(state: State<DbState>) -> Result<Vec<ChannelList>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT id, name, source, is_default, filepath, last_fetched FROM channel_lists")
        .map_err(|e| e.to_string())?;
    let list_iter = stmt
        .query_map([], |row| {
            Ok(ChannelList {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                is_default: row.get(3)?,
                filepath: row.get(4)?,
                last_fetched: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut lists = Vec::new();
    for list in list_iter {
        lists.push(list.map_err(|e| e.to_string())?);
    }
    Ok(lists)
}

#[tauri::command]
pub fn add_channel_list(state: State<DbState>, name: String, source: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
        &[&name, &source],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_default_channel_list(state: State<DbState>, id: i32) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let tx = db.transaction().map_err(|e| e.to_string())?;
    tx.execute("UPDATE channel_lists SET is_default = 0", [])
        .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE channel_lists SET is_default = 1 WHERE id = ?1",
        &[&id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: i32,
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute("DELETE FROM channel_lists WHERE id = ?1", &[&id])
        .map_err(|e| e.to_string())?;
    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub fn update_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: i32,
    name: String,
    source: String,
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute(
        "UPDATE channel_lists SET name = ?1, source = ?2 WHERE id = ?3",
        &[&name, &source, &id.to_string()],
    )
    .map_err(|e| e.to_string())?;
    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub fn start_channel_list_selection(cache_state: State<ChannelCacheState>) -> Result<(), String> {
    invalidate_channel_cache(cache_state)?;
    Ok(())
}
