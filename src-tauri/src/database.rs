use crate::error::{Result, TolloError};
use crate::m3u_parser::Channel;
use rusqlite::{Connection, Result as RusqliteResult};
use std::fs;

pub fn initialize_database() -> Result<Connection> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| TolloError::DataDirectoryAccess)?
        .join("tollo");

    fs::create_dir_all(&data_dir)
        .map_err(|_e| TolloError::directory_creation(data_dir.display().to_string()))?;

    let db_path = data_dir.join("database.sqlite");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS channels_fts USING fts5(name, content='channels', content_rowid='id')",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            player_command TEXT NOT NULL,
            cache_duration_hours INTEGER NOT NULL DEFAULT 24,
            enable_preview BOOLEAN NOT NULL DEFAULT 1,
            mute_on_start BOOLEAN NOT NULL DEFAULT 0,
            show_controls BOOLEAN NOT NULL DEFAULT 1,
            autoplay BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Add the enable_preview column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN enable_preview BOOLEAN NOT NULL DEFAULT 1",
        [],
    )
    .ok(); // Use ok() to ignore error if column already exists

    // Add the mute_on_start column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN mute_on_start BOOLEAN NOT NULL DEFAULT 0",
        [],
    )
    .ok();
    // Add the show_controls column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN show_controls BOOLEAN NOT NULL DEFAULT 1",
        [],
    )
    .ok();
    // Add the autoplay column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN autoplay BOOLEAN NOT NULL DEFAULT 0",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channel_lists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            source TEXT NOT NULL,
            filepath TEXT,
            last_fetched INTEGER,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT is_default_check CHECK (is_default IN (0, 1))
        )",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_one_default_list ON channel_lists (is_default) WHERE is_default = 1",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS group_selections (
            channel_list_id INTEGER NOT NULL,
            group_name TEXT NOT NULL,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            PRIMARY KEY (channel_list_id, group_name),
            FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_filters (
            channel_list_id INTEGER NOT NULL,
            slot_number INTEGER NOT NULL CHECK (slot_number >= 0 AND slot_number <= 9),
            search_query TEXT NOT NULL DEFAULT '',
            selected_group TEXT,
            name TEXT NOT NULL DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (channel_list_id, slot_number),
            FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    let list_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM channel_lists", [], |row| row.get(0))?;
    if list_count == 0 {
        conn.execute(
            "INSERT INTO channel_lists (name, source, is_default) VALUES (?1, ?2, ?3)",
            &["iptv-org", "https://iptv-org.github.io/iptv/index.m3u", "1"],
        )?;
    }

    // Ensure we have a default settings record
    let settings_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;
    if settings_count == 0 {
        conn.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, 'mpv', 24, 1, 0, 1, 0)",
            [],
        )?;
    }

    Ok(conn)
}

pub fn populate_channels(conn: &mut Connection, channels: &[Channel]) -> RusqliteResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT OR IGNORE INTO channels (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")?;
        for channel in channels {
            stmt.execute(&[
                &channel.name,
                &channel.logo,
                &channel.url,
                &channel.group_title,
                &channel.tvg_id,
                &channel.resolution,
                &channel.extra_info,
            ])?;
        }
    }
    tx.commit()?;

    conn.execute(
        "INSERT INTO channels_fts(rowid, name) SELECT id, name FROM channels",
        [],
    )?;

    Ok(())
}

pub fn get_enabled_groups(conn: &Connection, channel_list_id: i64) -> RusqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT group_name FROM group_selections WHERE channel_list_id = ?1 AND is_enabled = 1",
    )?;
    let group_iter = stmt.query_map([channel_list_id], |row| Ok(row.get::<_, String>(0)?))?;

    let mut groups = Vec::new();
    for group in group_iter {
        groups.push(group?);
    }
    Ok(groups)
}

pub fn set_group_enabled(
    conn: &Connection,
    channel_list_id: i64,
    group_name: String,
    enabled: bool,
) -> RusqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)",
        (channel_list_id, group_name, enabled),
    )?;
    Ok(())
}

pub fn sync_channel_list_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    let tx = conn.transaction()?;

    // Get existing groups for this channel list
    let existing_groups = {
        let mut stmt =
            tx.prepare("SELECT group_name FROM group_selections WHERE channel_list_id = ?1")?;
        let group_iter = stmt.query_map([channel_list_id], |row| Ok(row.get::<_, String>(0)?))?;

        let mut groups = Vec::new();
        for group in group_iter {
            groups.push(group?);
        }
        groups
    };

    // Remove groups that no longer exist
    for existing_group in &existing_groups {
        if !groups.contains(existing_group) {
            tx.execute(
                "DELETE FROM group_selections WHERE channel_list_id = ?1 AND group_name = ?2",
                (channel_list_id, existing_group),
            )?;
        }
    }

    // Add new groups (enabled by default)
    for group in &groups {
        if !existing_groups.contains(group) {
            tx.execute(
                "INSERT INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)",
                (channel_list_id, group, true),
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn enable_all_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    // Enable all groups in a single transaction for much better performance
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare("INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)")?;
        for group in groups {
            stmt.execute((channel_list_id, group, true))?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn disable_all_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    // Disable all groups in a single transaction for much better performance
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare("INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)")?;
        for group in groups {
            stmt.execute((channel_list_id, group, false))?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SavedFilter {
    pub slot_number: i32,
    pub search_query: String,
    pub selected_group: Option<String>,
    pub name: String,
}

pub fn save_filter(
    conn: &Connection,
    channel_list_id: i64,
    slot_number: i32,
    search_query: String,
    selected_group: Option<String>,
    name: String,
) -> RusqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO saved_filters (channel_list_id, slot_number, search_query, selected_group, name) VALUES (?1, ?2, ?3, ?4, ?5)",
        (channel_list_id, slot_number, search_query, selected_group, name),
    )?;
    Ok(())
}

pub fn get_saved_filters(
    conn: &Connection,
    channel_list_id: i64,
) -> RusqliteResult<Vec<SavedFilter>> {
    let mut stmt = conn.prepare("SELECT slot_number, search_query, selected_group, name FROM saved_filters WHERE channel_list_id = ?1 ORDER BY slot_number")?;
    let filter_iter = stmt.query_map([channel_list_id], |row| {
        Ok(SavedFilter {
            slot_number: row.get(0)?,
            search_query: row.get(1)?,
            selected_group: row.get(2)?,
            name: row.get(3)?,
        })
    })?;

    let mut filters = Vec::new();
    for filter in filter_iter {
        filters.push(filter?);
    }
    Ok(filters)
}

pub fn delete_saved_filter(
    conn: &Connection,
    channel_list_id: i64,
    slot_number: i32,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM saved_filters WHERE channel_list_id = ?1 AND slot_number = ?2",
        (channel_list_id, slot_number),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3u_parser::Channel;
    use rusqlite::Connection;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create the basic table structure that initialize_database would create
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                logo TEXT NOT NULL,
                url TEXT NOT NULL,
                group_title TEXT NOT NULL,
                tvg_id TEXT NOT NULL,
                resolution TEXT NOT NULL,
                extra_info TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                logo TEXT NOT NULL,
                url TEXT NOT NULL,
                group_title TEXT NOT NULL,
                tvg_id TEXT NOT NULL,
                resolution TEXT NOT NULL,
                extra_info TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS channels (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                logo TEXT NOT NULL,
                url TEXT NOT NULL,
                group_title TEXT NOT NULL,
                tvg_id TEXT NOT NULL,
                resolution TEXT NOT NULL,
                extra_info TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS channels_fts USING fts5(name, content='channels', content_rowid='id')",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS channel_lists (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                source TEXT NOT NULL,
                filepath TEXT,
                last_fetched INTEGER,
                is_default BOOLEAN NOT NULL DEFAULT 0,
                CONSTRAINT is_default_check CHECK (is_default IN (0, 1))
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS group_selections (
                channel_list_id INTEGER NOT NULL,
                group_name TEXT NOT NULL,
                is_enabled BOOLEAN NOT NULL DEFAULT 1,
                PRIMARY KEY (channel_list_id, group_name),
                FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS saved_filters (
                channel_list_id INTEGER NOT NULL,
                slot_number INTEGER NOT NULL CHECK (slot_number >= 0 AND slot_number <= 9),
                search_query TEXT NOT NULL DEFAULT '',
                selected_group TEXT,
                name TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (channel_list_id, slot_number),
                FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY,
                player_command TEXT NOT NULL,
                cache_duration_hours INTEGER NOT NULL DEFAULT 24,
                enable_preview BOOLEAN NOT NULL DEFAULT 1,
                mute_on_start BOOLEAN NOT NULL DEFAULT 0,
                show_controls BOOLEAN NOT NULL DEFAULT 1,
                autoplay BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )
        .unwrap();

        conn
    }

    fn create_test_channel() -> Channel {
        Channel {
            name: "Test Channel".to_string(),
            logo: "http://example.com/logo.png".to_string(),
            url: "http://example.com/stream".to_string(),
            group_title: "Test Group".to_string(),
            tvg_id: "test123".to_string(),
            resolution: "1080p".to_string(),
            extra_info: "Test extra info".to_string(),
        }
    }

    #[test]
    fn test_populate_channels_success() {
        let mut conn = create_test_db();
        let channels = vec![create_test_channel()];

        let result = populate_channels(&mut conn, &channels);
        assert!(result.is_ok());

        // Verify channel was inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Verify channel data
        let (name, url): (String, String) = conn
            .query_row("SELECT name, url FROM channels WHERE id = 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(name, "Test Channel");
        assert_eq!(url, "http://example.com/stream");
    }

    #[test]
    fn test_populate_channels_duplicate_handling() {
        let mut conn = create_test_db();
        let channels = vec![create_test_channel(), create_test_channel()];

        let result = populate_channels(&mut conn, &channels);
        assert!(result.is_ok());

        // Should only have one channel due to UNIQUE constraint on name
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_populate_channels_empty_list() {
        let mut conn = create_test_db();
        let channels = vec![];

        let result = populate_channels(&mut conn, &channels);
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_enabled_groups_empty() {
        let conn = create_test_db();

        let result = get_enabled_groups(&conn, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_set_and_get_enabled_groups() {
        let conn = create_test_db();

        // Insert a channel list first
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Set some groups as enabled
        set_group_enabled(&conn, 1, "Sports".to_string(), true).unwrap();
        set_group_enabled(&conn, 1, "News".to_string(), true).unwrap();
        set_group_enabled(&conn, 1, "Movies".to_string(), false).unwrap();

        let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
        assert_eq!(enabled_groups.len(), 2);
        assert!(enabled_groups.contains(&"Sports".to_string()));
        assert!(enabled_groups.contains(&"News".to_string()));
        assert!(!enabled_groups.contains(&"Movies".to_string()));
    }

    #[test]
    fn test_sync_channel_list_groups() {
        let mut conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Add some existing groups
        set_group_enabled(&conn, 1, "Sports".to_string(), true).unwrap();
        set_group_enabled(&conn, 1, "Old Group".to_string(), false).unwrap();

        // Sync with new groups (should remove "Old Group" and add "News")
        let new_groups = vec!["Sports".to_string(), "News".to_string()];
        sync_channel_list_groups(&mut conn, 1, new_groups).unwrap();

        let all_groups: Vec<String> = conn
            .prepare("SELECT group_name FROM group_selections WHERE channel_list_id = 1")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();

        assert_eq!(all_groups.len(), 2);
        assert!(all_groups.contains(&"Sports".to_string()));
        assert!(all_groups.contains(&"News".to_string()));
        assert!(!all_groups.contains(&"Old Group".to_string()));
    }

    #[test]
    fn test_enable_all_groups() {
        let mut conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        let groups = vec![
            "Sports".to_string(),
            "News".to_string(),
            "Movies".to_string(),
        ];
        enable_all_groups(&mut conn, 1, groups).unwrap();

        let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
        assert_eq!(enabled_groups.len(), 3);
        assert!(enabled_groups.contains(&"Sports".to_string()));
        assert!(enabled_groups.contains(&"News".to_string()));
        assert!(enabled_groups.contains(&"Movies".to_string()));
    }

    #[test]
    fn test_save_and_get_saved_filters() {
        let conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Save some filters
        save_filter(
            &conn,
            1,
            0,
            "test search".to_string(),
            Some("Sports".to_string()),
            "My Filter".to_string(),
        )
        .unwrap();
        save_filter(
            &conn,
            1,
            1,
            "another search".to_string(),
            None,
            "Another Filter".to_string(),
        )
        .unwrap();

        let filters = get_saved_filters(&conn, 1).unwrap();
        assert_eq!(filters.len(), 2);

        let filter0 = &filters[0];
        assert_eq!(filter0.slot_number, 0);
        assert_eq!(filter0.search_query, "test search");
        assert_eq!(filter0.selected_group, Some("Sports".to_string()));
        assert_eq!(filter0.name, "My Filter");

        let filter1 = &filters[1];
        assert_eq!(filter1.slot_number, 1);
        assert_eq!(filter1.search_query, "another search");
        assert_eq!(filter1.selected_group, None);
        assert_eq!(filter1.name, "Another Filter");
    }

    #[test]
    fn test_delete_saved_filter() {
        let conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Save some filters
        save_filter(
            &conn,
            1,
            0,
            "test search".to_string(),
            Some("Sports".to_string()),
            "My Filter".to_string(),
        )
        .unwrap();
        save_filter(
            &conn,
            1,
            1,
            "another search".to_string(),
            None,
            "Another Filter".to_string(),
        )
        .unwrap();

        // Delete one filter
        delete_saved_filter(&conn, 1, 0).unwrap();

        let filters = get_saved_filters(&conn, 1).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].slot_number, 1);
    }

    #[test]
    fn test_saved_filter_replace() {
        let conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Save a filter
        save_filter(
            &conn,
            1,
            0,
            "original search".to_string(),
            Some("Sports".to_string()),
            "Original".to_string(),
        )
        .unwrap();

        // Replace it with a new filter
        save_filter(
            &conn,
            1,
            0,
            "new search".to_string(),
            Some("News".to_string()),
            "New Filter".to_string(),
        )
        .unwrap();

        let filters = get_saved_filters(&conn, 1).unwrap();
        assert_eq!(filters.len(), 1);

        let filter = &filters[0];
        assert_eq!(filter.search_query, "new search");
        assert_eq!(filter.selected_group, Some("News".to_string()));
        assert_eq!(filter.name, "New Filter");
    }

    #[test]
    fn test_saved_filter_slot_constraints() {
        let conn = create_test_db();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Valid slot numbers (0-9)
        for slot in 0..=9 {
            let result = save_filter(&conn, 1, slot, "test".to_string(), None, "Test".to_string());
            assert!(result.is_ok(), "Slot {} should be valid", slot);
        }

        // Invalid slot numbers should fail (this would be caught by CHECK constraint)
        // Note: The actual constraint check happens at the database level
    }

    #[test]
    fn test_foreign_key_constraints() {
        let conn = create_test_db();

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Try to save a filter for a non-existent channel list
        let result = save_filter(&conn, 999, 0, "test".to_string(), None, "Test".to_string());
        assert!(result.is_err(), "Should fail due to foreign key constraint");

        // Try to set group for a non-existent channel list
        let result = set_group_enabled(&conn, 999, "Sports".to_string(), true);
        assert!(result.is_err(), "Should fail due to foreign key constraint");
    }

    #[test]
    fn test_cascade_delete() {
        let conn = create_test_db();

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Insert a channel list
        conn.execute(
            "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
            [],
        ).unwrap();

        // Add some dependent data
        set_group_enabled(&conn, 1, "Sports".to_string(), true).unwrap();
        save_filter(&conn, 1, 0, "test".to_string(), None, "Test".to_string()).unwrap();

        // Delete the channel list
        conn.execute("DELETE FROM channel_lists WHERE id = 1", [])
            .unwrap();

        // Verify dependent data was deleted
        let group_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM group_selections WHERE channel_list_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(group_count, 0);

        let filter_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM saved_filters WHERE channel_list_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(filter_count, 0);
    }

    // Error scenario tests
    mod error_tests {
        use super::*;

        #[test]
        fn test_populate_channels_with_invalid_transaction() {
            let mut conn = create_test_db();

            // Create an empty channel with all required fields
            let invalid_channel = Channel {
                name: "".to_string(), // Empty name might cause issues
                logo: "".to_string(),
                url: "".to_string(),
                group_title: "".to_string(),
                tvg_id: "".to_string(),
                resolution: "".to_string(),
                extra_info: "".to_string(),
            };

            let channels = vec![invalid_channel];
            let result = populate_channels(&mut conn, &channels);

            // Should still succeed as empty strings are valid in SQLite
            assert!(result.is_ok());
        }

        #[test]
        fn test_populate_channels_with_extremely_long_strings() {
            let mut conn = create_test_db();

            // Create channel with very long strings to test limits
            let long_string = "a".repeat(10000);
            let long_channel = Channel {
                name: long_string.clone(),
                logo: long_string.clone(),
                url: long_string.clone(),
                group_title: long_string.clone(),
                tvg_id: long_string.clone(),
                resolution: long_string.clone(),
                extra_info: long_string,
            };

            let channels = vec![long_channel];
            let result = populate_channels(&mut conn, &channels);

            // Should succeed as SQLite handles long strings
            assert!(result.is_ok());
        }

        #[test]
        fn test_populate_channels_with_special_characters() {
            let mut conn = create_test_db();

            // Create channel with special characters that might cause SQL injection
            let special_channel = Channel {
                name: "'; DROP TABLE channels; --".to_string(),
                logo: "http://example.com/logo'; --".to_string(),
                url: "http://example.com/stream'; --".to_string(),
                group_title: "Group'; --".to_string(),
                tvg_id: "test'; --".to_string(),
                resolution: "1080p'; --".to_string(),
                extra_info: "Extra'; --".to_string(),
            };

            let channels = vec![special_channel];
            let result = populate_channels(&mut conn, &channels);

            // Should succeed as we use prepared statements
            assert!(result.is_ok());

            // Verify table still exists and data was inserted safely
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 1);
        }

        #[test]
        fn test_get_enabled_groups_with_invalid_channel_list_id() {
            let conn = create_test_db();

            // Try to get groups for non-existent channel list
            let result = get_enabled_groups(&conn, -1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);

            let result = get_enabled_groups(&conn, 999999);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[test]
        fn test_set_group_enabled_with_very_long_group_name() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            let very_long_group_name = "a".repeat(1000);
            let result = set_group_enabled(&conn, 1, very_long_group_name.clone(), true);
            assert!(result.is_ok());

            let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
            assert_eq!(enabled_groups.len(), 1);
            assert_eq!(enabled_groups[0], very_long_group_name);
        }

        #[test]
        fn test_sync_channel_list_groups_with_empty_groups() {
            let mut conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            // Add some existing groups
            set_group_enabled(&conn, 1, "Sports".to_string(), true).unwrap();
            set_group_enabled(&conn, 1, "News".to_string(), false).unwrap();

            // Sync with empty groups list (should remove all existing groups)
            let empty_groups = vec![];
            let result = sync_channel_list_groups(&mut conn, 1, empty_groups);
            assert!(result.is_ok());

            let remaining_groups = get_enabled_groups(&conn, 1).unwrap();
            assert_eq!(remaining_groups.len(), 0);
        }

        #[test]
        fn test_enable_all_groups_with_empty_list() {
            let mut conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            let empty_groups = vec![];
            let result = enable_all_groups(&mut conn, 1, empty_groups);
            assert!(result.is_ok());

            let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
            assert_eq!(enabled_groups.len(), 0);
        }

        #[test]
        fn test_save_filter_with_invalid_slot_numbers() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            // Test boundary values for slot numbers
            let result = save_filter(&conn, 1, -1, "test".to_string(), None, "Test".to_string());
            assert!(
                result.is_err(),
                "Negative slot should fail CHECK constraint"
            );

            let result = save_filter(&conn, 1, 10, "test".to_string(), None, "Test".to_string());
            assert!(result.is_err(), "Slot > 9 should fail CHECK constraint");
        }

        #[test]
        fn test_save_filter_with_extremely_long_values() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            let very_long_string = "a".repeat(10000);
            let result = save_filter(
                &conn,
                1,
                0,
                very_long_string.clone(),
                Some(very_long_string.clone()),
                very_long_string,
            );
            assert!(result.is_ok());
        }

        #[test]
        fn test_get_saved_filters_with_invalid_channel_list_id() {
            let conn = create_test_db();

            let result = get_saved_filters(&conn, -1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);

            let result = get_saved_filters(&conn, 999999);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[test]
        fn test_delete_saved_filter_nonexistent() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            // Try to delete non-existent filter
            let result = delete_saved_filter(&conn, 1, 0);
            assert!(result.is_ok()); // Should succeed even if nothing to delete

            // Try to delete from non-existent channel list
            let result = delete_saved_filter(&conn, 999, 0);
            assert!(result.is_ok()); // Should succeed even if nothing to delete
        }

        #[test]
        fn test_concurrent_operations_simulation() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            // Simulate concurrent group operations
            for i in 0..100 {
                let group_name = format!("Group{}", i);
                let result = set_group_enabled(&conn, 1, group_name, i % 2 == 0);
                assert!(result.is_ok());
            }

            let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
            assert_eq!(enabled_groups.len(), 50); // Half should be enabled
        }

        #[test]
        fn test_unicode_and_emoji_handling() {
            let conn = create_test_db();

            // Insert a channel list
            conn.execute(
                "INSERT INTO channel_lists (id, name, source) VALUES (1, 'Test List', 'http://example.com')",
                [],
            ).unwrap();

            // Test with various Unicode characters and emojis
            let unicode_cases = vec![
                "Спорт 🏀",            // Russian + emoji
                "Canal Español 📺",    // Spanish + emoji
                "Chaîne Française 🇫🇷", // French + flag emoji
                "中文频道 📡",         // Chinese + emoji
                "قناة العربية 🌙",     // Arabic + emoji
                "🎬📺🎪🎭🎨",          // Multiple emojis
            ];

            for (i, group_name) in unicode_cases.iter().enumerate() {
                let result = set_group_enabled(&conn, 1, group_name.to_string(), true);
                assert!(
                    result.is_ok(),
                    "Failed to handle Unicode group: {}",
                    group_name
                );

                // Also test filters with Unicode
                let result = save_filter(
                    &conn,
                    1,
                    i as i32,
                    format!("search {}", group_name),
                    Some(group_name.to_string()),
                    format!("Filter {}", group_name),
                );
                assert!(
                    result.is_ok(),
                    "Failed to save Unicode filter: {}",
                    group_name
                );
            }

            let enabled_groups = get_enabled_groups(&conn, 1).unwrap();
            assert_eq!(enabled_groups.len(), unicode_cases.len());

            let filters = get_saved_filters(&conn, 1).unwrap();
            assert_eq!(filters.len(), unicode_cases.len());
        }
    }
}
