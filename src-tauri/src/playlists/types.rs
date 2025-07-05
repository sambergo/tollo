use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistFetchStatus {
    pub id: i32,
    pub status: String, // "starting", "fetching", "processing", "saving", "completed", "error"
    pub progress: f32,  // 0.0 to 1.0
    pub message: String,
    pub channel_count: Option<usize>,
    pub error: Option<String>,
}

pub struct FetchState {
    pub operations: Arc<AsyncMutex<HashMap<i32, PlaylistFetchStatus>>>,
}

impl FetchState {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}

// Helper function to emit progress events
pub async fn emit_progress(
    app_handle: &AppHandle,
    fetch_state: &State<'_, FetchState>,
    status: PlaylistFetchStatus,
) {
    // Update the state
    let mut operations = fetch_state.operations.lock().await;
    operations.insert(status.id, status.clone());
    drop(operations);

    // Emit event to frontend
    if let Err(e) = app_handle.emit("playlist_fetch_status", &status) {
        eprintln!("Failed to emit playlist_fetch_status event: {}", e);
    }
}
