use crate::error::{Result as TolloResult, TolloError};
use std::fs;
use rusqlite::Connection;
use dirs;

// Add cleanup function near the top with other utility functions
pub fn cleanup_orphaned_channel_files(db_connection: &Connection) -> TolloResult<()> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| TolloError::DataDirectoryAccess)?
        .join("tollo");
    let channel_lists_dir = data_dir.join("channel_lists");
    
    // Create channel_lists directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&channel_lists_dir) {
        println!("Warning: Failed to create channel_lists directory: {}", e);
        return Ok(()); // Don't fail startup if we can't create the directory
    }
    
    // Get all .m3u files in the channel_lists directory
    let disk_files = match fs::read_dir(&channel_lists_dir) {
        Ok(entries) => {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().is_file() && 
                    entry.path().extension().map_or(false, |ext| ext == "m3u")
                })
                .filter_map(|entry| {
                    entry.file_name().to_str().map(|s| s.to_string())
                })
                .collect::<Vec<String>>()
        },
        Err(_) => {
            println!("Channel lists directory not found or inaccessible, skipping cleanup");
            return Ok(());
        }
    };
    
    // Get all filepaths from database
    let mut stmt = match db_connection.prepare("SELECT filepath FROM channel_lists WHERE filepath IS NOT NULL") {
        Ok(stmt) => stmt,
        Err(e) => {
            println!("Warning: Failed to prepare database query for cleanup: {}", e);
            return Ok(());
        }
    };
    
    let db_files: Result<Vec<String>, rusqlite::Error> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).and_then(|iter| iter.collect());
    
    let db_files = match db_files {
        Ok(files) => files,
        Err(e) => {
            println!("Warning: Failed to query database for cleanup: {}", e);
            return Ok(());
        }
    };
    
    // Find orphaned files (on disk but not in database)
    let orphaned_files: Vec<String> = disk_files
        .into_iter()
        .filter(|disk_file| !db_files.contains(disk_file))
        .collect();
    
    let mut deleted_count = 0;
    let mut failed_deletions = 0;
    
    // Delete orphaned files
    for filename in &orphaned_files {
        let file_path = channel_lists_dir.join(filename);
        match fs::remove_file(&file_path) {
            Ok(_) => {
                deleted_count += 1;
                println!("Deleted orphaned channel list file: {}", filename);
            },
            Err(e) => {
                failed_deletions += 1;
                println!("Failed to delete orphaned file {}: {}", filename, e);
            }
        }
    }
    
    // Log cleanup statistics
    if deleted_count > 0 || failed_deletions > 0 {
        println!("Channel list cache cleanup completed: {} files deleted, {} failures", 
                deleted_count, failed_deletions);
    } else if !orphaned_files.is_empty() {
        println!("Channel list cache cleanup: no orphaned files found");
    }
    
    Ok(())
} 