use crate::{
    database::SavedFilter,
    m3u_parser::Channel,
    operation_gate,
    state::{ChannelCacheState, DbState},
};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

const FORMAT: &str = "tollo-user-data";
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct BackupSettings {
    player_command: String,
    clipboard_command: String,
    cache_duration_hours: i64,
    enable_preview: bool,
    mute_on_start: bool,
    show_controls: bool,
    autoplay: bool,
}

#[derive(Serialize, Deserialize)]
struct GroupSelection {
    name: String,
    enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct BackupList {
    name: String,
    source: String,
    is_default: bool,
    local_content: Option<String>,
    saved_filters: Vec<SavedFilter>,
    groups: Vec<GroupSelection>,
}

#[derive(Serialize, Deserialize)]
struct Backup {
    format: String,
    version: u32,
    exported_at: String,
    app_version: String,
    settings: BackupSettings,
    favorites: Vec<Channel>,
    channel_lists: Vec<BackupList>,
}

#[derive(Serialize)]
pub struct ImportPreview {
    token: String,
    exported_at: String,
    app_version: String,
    settings: BackupSettings,
    list_names: Vec<String>,
    default_list: Option<String>,
    favorite_count: usize,
    filter_count: usize,
    group_count: usize,
    local_playlist_count: usize,
}

// A single retained preview bounds memory and makes older tokens invalid.
#[derive(Default)]
pub struct BackupState {
    pending: Mutex<Option<(String, Vec<u8>)>>,
}

fn remote(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

fn valid_m3u(content: &str) -> bool {
    let mut lines = content.trim_start_matches('\u{feff}').trim_start().lines();
    if lines.next().and_then(|line| line.split_whitespace().next()) != Some("#EXTM3U") {
        return false;
    }
    let mut pending_entry = false;
    for line in lines.map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with("#EXTINF:") {
            if pending_entry || !line.contains(',') {
                return false;
            }
            pending_entry = true;
        } else if !line.starts_with('#') {
            pending_entry = false;
        }
    }
    !pending_entry
}

fn validate(backup: &Backup) -> Result<(), String> {
    if backup.format != FORMAT || backup.version != VERSION {
        return Err("Unsupported Tollo backup format or version".into());
    }
    chrono::DateTime::parse_from_rfc3339(&backup.exported_at)
        .map_err(|_| "Invalid export timestamp")?;
    if backup.app_version.trim().is_empty()
        || backup.settings.player_command.trim().is_empty()
        || backup.settings.clipboard_command.trim().is_empty()
        || !(1..=i64::MAX / 3600).contains(&backup.settings.cache_duration_hours)
    {
        return Err("Invalid backup settings or app version".into());
    }
    let mut names = HashSet::new();
    for favorite in &backup.favorites {
        if favorite.name.trim().is_empty()
            || favorite.url.trim().is_empty()
            || !names.insert(&favorite.name)
        {
            return Err("Favorites must have unique nonempty names and stream URLs".into());
        }
    }
    names.clear();
    for list in &backup.channel_lists {
        if list.name.trim().is_empty() || list.source.trim().is_empty() || !names.insert(&list.name)
        {
            return Err("Channel lists must have unique nonempty names and sources".into());
        }
        match (&list.local_content, remote(&list.source)) {
            (None, true) => {}
            (Some(content), false) if valid_m3u(content) => {}
            _ => {
                return Err(format!(
                    "Invalid embedded playlist or source for '{}'",
                    list.name
                ))
            }
        }
        let mut slots = HashSet::new();
        for filter in &list.saved_filters {
            if !(0..=9).contains(&filter.slot_number) || !slots.insert(filter.slot_number) {
                return Err(format!(
                    "Invalid or duplicate filter slot in '{}'",
                    list.name
                ));
            }
        }
        let mut groups = HashSet::new();
        for group in &list.groups {
            if !groups.insert(&group.name) {
                return Err(format!("Duplicate group selection in '{}'", list.name));
            }
        }
    }
    if backup
        .channel_lists
        .iter()
        .filter(|list| list.is_default)
        .count()
        > 1
    {
        return Err("A backup cannot contain multiple default channel lists".into());
    }
    Ok(())
}

fn parse(bytes: &[u8]) -> Result<Backup, String> {
    let backup: Backup =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid backup JSON: {e}"))?;
    validate(&backup)?;
    Ok(backup)
}

fn snapshot(db: &Connection) -> Result<Backup, String> {
    let settings = db.query_row("SELECT player_command, clipboard_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay FROM settings WHERE id = 1", [], |row| Ok(BackupSettings {
        player_command: row.get(0)?, clipboard_command: row.get(1)?, cache_duration_hours: row.get(2)?,
        enable_preview: row.get(3)?, mute_on_start: row.get(4)?, show_controls: row.get(5)?, autoplay: row.get(6)?,
    })).map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT name, logo, url, group_title, tvg_id, resolution, extra_info FROM favorites ORDER BY position, id").map_err(|e| e.to_string())?;
    let favorites = stmt
        .query_map([], |r| {
            Ok(Channel {
                name: r.get(0)?,
                logo: r.get(1)?,
                url: r.get(2)?,
                group_title: r.get(3)?,
                tvg_id: r.get(4)?,
                resolution: r.get(5)?,
                extra_info: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, name, source, is_default FROM channel_lists ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut channel_lists = Vec::new();
    for (id, name, source, is_default) in rows {
        let local_content = if remote(&source) {
            None
        } else {
            Some(
                fs::read_to_string(&source)
                    .map_err(|e| format!("Cannot read local playlist '{name}': {e}"))?,
            )
        };
        let saved_filters =
            crate::database::get_saved_filters(db, id).map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT group_name, is_enabled FROM group_selections WHERE channel_list_id = ?1 ORDER BY group_name").map_err(|e| e.to_string())?;
        let groups = stmt
            .query_map([id], |r| {
                Ok(GroupSelection {
                    name: r.get(0)?,
                    enabled: r.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        channel_lists.push(BackupList {
            name,
            source,
            is_default,
            local_content,
            saved_filters,
            groups,
        });
    }
    let backup = Backup {
        format: FORMAT.into(),
        version: VERSION,
        exported_at: Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        settings,
        favorites,
        channel_lists,
    };
    validate(&backup)?;
    Ok(backup)
}

fn atomic_save(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid export path")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| format!("Cannot create backup: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("Cannot write backup: {e}"))?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path)
        .map_err(|e| format!("Cannot save backup: {e}"))?;
    Ok(())
}

fn restore(db: &mut Connection, backup: &Backup, data_dir: &Path) -> Result<(), String> {
    validate(backup)?;
    // Unique source files live outside the disposable playlist cache directory.
    let playlist_dir = data_dir.join("imported_playlists");
    let mut staged = Vec::<PathBuf>::new();
    let result = (|| {
        let mut sources = Vec::new();
        for list in &backup.channel_lists {
            if let Some(content) = &list.local_content {
                fs::create_dir_all(&playlist_dir).map_err(|e| e.to_string())?;
                let path = playlist_dir.join(format!("{}.m3u", Uuid::new_v4()));
                atomic_save(&path, content.as_bytes())?;
                sources.push(path.to_string_lossy().into_owned());
                staged.push(path);
            } else {
                sources.push(list.source.clone());
            }
        }
        let tx = db.transaction().map_err(|e| e.to_string())?;
        let max_id: i64 = tx
            .query_row("SELECT COALESCE(MAX(id), 0) FROM channel_lists", [], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        for table in [
            "saved_filters",
            "group_selections",
            "channel_lists",
            "favorites",
        ] {
            tx.execute(&format!("DELETE FROM {table}"), [])
                .map_err(|e| e.to_string())?;
        }
        let s = &backup.settings;
        tx.execute("INSERT OR REPLACE INTO settings (id, player_command, clipboard_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![s.player_command, s.clipboard_command, s.cache_duration_hours, s.enable_preview, s.mute_on_start, s.show_controls, s.autoplay]).map_err(|e| e.to_string())?;
        for (index, (list, source)) in backup.channel_lists.iter().zip(sources).enumerate() {
            let id = max_id
                .checked_add(index as i64 + 1)
                .filter(|id| *id <= i32::MAX as i64)
                .ok_or("Channel list ID limit reached")?;
            tx.execute("INSERT INTO channel_lists (id, name, source, is_default, filepath, last_fetched) VALUES (?1, ?2, ?3, ?4, NULL, NULL)", params![id, list.name, source, list.is_default]).map_err(|e| e.to_string())?;
            for f in &list.saved_filters {
                tx.execute("INSERT INTO saved_filters (channel_list_id, slot_number, search_query, selected_group, name) VALUES (?1, ?2, ?3, ?4, ?5)", params![id, f.slot_number, f.search_query, f.selected_group, f.name]).map_err(|e| e.to_string())?;
            }
            for g in &list.groups {
                tx.execute("INSERT INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)", params![id, g.name, g.enabled]).map_err(|e| e.to_string())?;
            }
        }
        for (position, c) in backup.favorites.iter().enumerate() {
            tx.execute("INSERT INTO favorites (name, logo, url, group_title, tvg_id, resolution, extra_info, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![c.name, c.logo, c.url, c.group_title, c.tvg_id, c.resolution, c.extra_info, position as i64]).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    })();
    if result.is_err() {
        for path in staged {
            let _ = fs::remove_file(path);
        }
    }
    result
}

impl BackupState {
    fn preview(&self, bytes: Vec<u8>) -> Result<ImportPreview, String> {
        let backup = parse(&bytes)?;
        let token = Uuid::new_v4().to_string();
        let preview = ImportPreview {
            token: token.clone(),
            exported_at: backup.exported_at,
            app_version: backup.app_version,
            default_list: backup
                .channel_lists
                .iter()
                .find(|l| l.is_default)
                .map(|l| l.name.clone()),
            list_names: backup
                .channel_lists
                .iter()
                .map(|l| l.name.clone())
                .collect(),
            favorite_count: backup.favorites.len(),
            filter_count: backup
                .channel_lists
                .iter()
                .map(|l| l.saved_filters.len())
                .sum(),
            group_count: backup.channel_lists.iter().map(|l| l.groups.len()).sum(),
            local_playlist_count: backup
                .channel_lists
                .iter()
                .filter(|l| l.local_content.is_some())
                .count(),
            settings: backup.settings,
        };
        *self.pending.lock().map_err(|e| e.to_string())? = Some((token, bytes));
        Ok(preview)
    }

    fn take(&self, token: &str) -> Result<Backup, String> {
        let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
        if pending.as_ref().map(|p| p.0.as_str()) != Some(token) {
            return Err("Import preview expired; select the backup again".into());
        }
        let (_, bytes) = pending.take().unwrap();
        parse(&bytes)
    }

    fn cancel(&self, token: &str) -> Result<(), String> {
        let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
        if pending.as_ref().map(|p| p.0.as_str()) == Some(token) {
            pending.take();
        }
        Ok(())
    }
}

#[tauri::command]
pub async fn export_user_data(app: AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Snapshot before opening the dialog, releasing database and workflow locks promptly.
        let bytes = {
            let _operation = operation_gate::write()?;
            let state = app.state::<DbState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            serde_json::to_vec_pretty(&snapshot(&db)?).map_err(|e| e.to_string())?
        };
        let filename = format!("tollo-backup-{}.json", Utc::now().format("%Y-%m-%d"));
        let Some(path) = app
            .dialog()
            .file()
            .add_filter("Tollo backup", &["json"])
            .set_file_name(filename)
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        atomic_save(&path, &bytes)?;
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn preview_user_data_import(app: AppHandle) -> Result<Option<ImportPreview>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = app
            .dialog()
            .file()
            .add_filter("Tollo backup", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        let bytes = fs::read(path).map_err(|e| format!("Cannot read backup: {e}"))?;
        app.state::<BackupState>().preview(bytes).map(Some)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn confirm_user_data_import(app: AppHandle, token: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation_gate::write()?;
        let backup = app.state::<BackupState>().take(&token)?;
        let data_dir = dirs::data_dir()
            .ok_or("Cannot locate application data directory")?
            .join("tollo");
        let state = app.state::<DbState>();
        let cache_state = app.state::<ChannelCacheState>();
        // Follow channel loading's cache-before-database lock order. Acquire before commit
        // so a poisoned cache cannot turn a successful restore into a reported failure.
        let mut cache = cache_state.cache.lock().map_err(|e| e.to_string())?;
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        restore(&mut db, &backup, &data_dir)?;
        *cache = None;
        crate::search::clear_advanced_cache();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn cancel_user_data_import(state: State<BackupState>, token: String) -> Result<(), String> {
    state.cancel(&token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn db() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        crate::database::initialize_schema(&db).unwrap();
        db
    }

    fn populated(db: &Connection, dir: &Path) -> Backup {
        let path = dir.join("original.m3u");
        fs::write(
            &path,
            "#EXTM3U\n#EXTINF:-1 group-title=\"News\",Local\nhttps://example.com/stream\n",
        )
        .unwrap();
        db.execute(
            "INSERT INTO channel_lists (id, name, source, is_default) VALUES (10, 'Local', ?1, 0)",
            [path.to_string_lossy().as_ref()],
        )
        .unwrap();
        db.execute(
            "UPDATE channel_lists SET filepath = 'old-cache.m3u', last_fetched = 123 WHERE id = 1",
            [],
        )
        .unwrap();
        db.execute("INSERT INTO group_selections VALUES (10, 'News', 0)", [])
            .unwrap();
        db.execute("INSERT INTO saved_filters (channel_list_id, slot_number, search_query, selected_group, name) VALUES (10, 3, 'test', 'News', 'My filter')", []).unwrap();
        for (name, position) in [("Second", 1), ("First", 0)] {
            db.execute("INSERT INTO favorites (name, logo, url, group_title, tvg_id, resolution, extra_info, position) VALUES (?1, '', 'https://example.com/stream', 'News', '', '1080p', '', ?2)", params![name, position]).unwrap();
        }
        snapshot(db).unwrap()
    }

    #[test]
    fn round_trip_preserves_settings_order_and_relationships_without_original_file() {
        let dir = tempdir().unwrap();
        let mut db = db();
        let backup = populated(&db, dir.path());
        let bytes = serde_json::to_vec(&backup).unwrap();
        fs::remove_file(dir.path().join("original.m3u")).unwrap();
        db.execute("INSERT INTO history (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES ('Watched', '', 'stream', '', '', '', '')", []).unwrap();
        restore(&mut db, &parse(&bytes).unwrap(), dir.path()).unwrap();
        let restored = snapshot(&db).unwrap();
        assert_eq!(
            restored
                .favorites
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["First", "Second"]
        );
        assert_eq!(
            serde_json::to_value(&restored.settings).unwrap(),
            serde_json::to_value(&backup.settings).unwrap()
        );
        let local = restored
            .channel_lists
            .iter()
            .find(|l| l.name == "Local")
            .unwrap();
        assert_eq!(local.local_content, backup.channel_lists[1].local_content);
        assert_eq!(local.saved_filters[0].slot_number, 3);
        assert_eq!(
            local.saved_filters[0].selected_group.as_deref(),
            Some("News")
        );
        assert!(!local.groups[0].enabled);
        assert!(restored.channel_lists[0].is_default);
        assert!(Path::new(&local.source).exists());
        let local_id = db
            .query_row(
                "SELECT id FROM channel_lists WHERE name = 'Local'",
                [],
                |r| r.get::<_, i32>(0),
            )
            .unwrap();
        let content = crate::m3u_parser_helpers::get_m3u_content(&mut db, Some(local_id)).unwrap();
        assert_eq!(
            crate::m3u_parser_helpers::parse_m3u_with_progress(&content, |_, _, _| {}).len(),
            1
        );
        assert!(db
            .query_row("SELECT MIN(id) > 10 FROM channel_lists", [], |r| r
                .get::<_, bool>(0))
            .unwrap());
        assert_eq!(db.query_row("SELECT COUNT(*) FROM channel_lists WHERE filepath IS NOT NULL OR last_fetched IS NOT NULL", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(
            db.query_row("SELECT name FROM history", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "Watched"
        );
    }

    #[test]
    fn empty_collections_restore() {
        let mut db = db();
        let mut backup = snapshot(&db).unwrap();
        backup.channel_lists.clear();
        let dir = tempdir().unwrap();
        restore(&mut db, &backup, dir.path()).unwrap();
        crate::database::initialize_schema(&db).unwrap();
        let restored = snapshot(&db).unwrap();
        assert!(restored.channel_lists.is_empty());
        assert!(restored.favorites.is_empty());
    }

    #[test]
    fn rejects_malformed_unsupported_and_incomplete_json() {
        assert!(parse(b"{broken").is_err());
        assert!(parse(b"{}").is_err());
        let mut backup = snapshot(&db()).unwrap();
        backup.version = 99;
        assert!(parse(&serde_json::to_vec(&backup).unwrap()).is_err());
        backup.version = VERSION;
        backup.format = "another-app".into();
        assert!(validate(&backup).is_err());
    }

    #[test]
    fn rejects_duplicate_names_filters_groups_defaults_and_invalid_m3u() {
        let db = db();
        let dir = tempdir().unwrap();
        let backup = populated(&db, dir.path());
        let original = serde_json::to_vec(&backup).unwrap();
        for case in 0..8 {
            let mut b = parse(&original).unwrap();
            match case {
                0 => b.favorites[1].name = b.favorites[0].name.clone(),
                1 => b.channel_lists[1].name = b.channel_lists[0].name.clone(),
                2 => b.channel_lists[1].saved_filters[0].slot_number = 10,
                3 => b.channel_lists[1].saved_filters.push(SavedFilter {
                    slot_number: 3,
                    search_query: String::new(),
                    selected_group: None,
                    name: String::new(),
                }),
                4 => b.channel_lists[1].groups.push(GroupSelection {
                    name: "News".into(),
                    enabled: true,
                }),
                5 => b.channel_lists[1].is_default = true,
                6 => b.channel_lists[1].local_content = Some("not M3U".into()),
                _ => b.channel_lists[1].local_content = None,
            }
            assert!(validate(&b).is_err(), "case {case}");
        }
    }

    #[test]
    fn playlist_validation_accepts_header_attributes_and_rejects_unfinished_entries() {
        assert!(valid_m3u("#EXTM3U x-tvg-url=\"https://example.com/epg\"\n#EXTINF:-1,Test\nhttps://example.com/stream\n"));
        assert!(valid_m3u("#EXTM3U\n"));
        assert!(!valid_m3u("#EXTM3U\n#EXTINF:-1,Test\n"));
        assert!(!valid_m3u(
            "#EXTM3U\n#EXTINF:-1,Test\n#EXTINF:-1,Another\nstream\n"
        ));
    }

    #[test]
    fn missing_local_file_prevents_export() {
        let db = db();
        db.execute("INSERT INTO channel_lists (name, source) VALUES ('Missing', '/does-not-exist/tollo.m3u')", []).unwrap();
        assert!(snapshot(&db).err().unwrap().contains("Missing"));
    }

    #[test]
    fn cancellation_invalidates_token_and_preview_retains_original_bytes() {
        let state = BackupState::default();
        let backup = snapshot(&db()).unwrap();
        let bytes = serde_json::to_vec(&backup).unwrap();
        let preview = state.preview(bytes.clone()).unwrap();
        assert!(state.take("wrong-token").is_err());
        state.cancel(&preview.token).unwrap();
        assert!(state.take(&preview.token).is_err());
        let first = state.preview(bytes.clone()).unwrap();
        let second = state.preview(bytes).unwrap();
        assert!(state.take(&first.token).is_err());
        let retained = state.take(&second.token).unwrap();
        assert_eq!(retained.exported_at, backup.exported_at);
        assert!(state.take(&second.token).is_err());
    }

    #[test]
    fn transaction_failure_preserves_data_and_removes_staged_files() {
        let dir = tempdir().unwrap();
        let mut db = db();
        let backup = populated(&db, dir.path());
        db.execute_batch("CREATE TRIGGER fail_restore BEFORE INSERT ON favorites BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        assert!(restore(&mut db, &backup, dir.path()).is_err());
        assert_eq!(
            fs::read_dir(dir.path().join("imported_playlists"))
                .unwrap()
                .count(),
            0
        );
        let after = snapshot(&db).unwrap();
        assert_eq!(
            after.channel_lists[1].source,
            backup.channel_lists[1].source
        );
        assert_eq!(after.favorites, backup.favorites);
        assert_eq!(
            db.query_row(
                "SELECT last_fetched FROM channel_lists WHERE id = 1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            123
        );
    }

    #[test]
    fn failed_file_staging_preserves_database_and_atomic_export_preserves_destination() {
        let dir = tempdir().unwrap();
        let mut db = db();
        let backup = populated(&db, dir.path());
        fs::write(dir.path().join("imported_playlists"), "blocking file").unwrap();
        assert!(restore(&mut db, &backup, dir.path()).is_err());
        assert_eq!(snapshot(&db).unwrap().favorites, backup.favorites);
        let destination = dir.path().join("backup.json");
        atomic_save(&destination, b"original").unwrap();
        atomic_save(&destination, b"replacement").unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"replacement");
        assert!(atomic_save(&dir.path().join("missing/backup.json"), b"bad").is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"replacement");
    }
}
