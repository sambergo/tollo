use tauri::ipc::InvokeError;

/// Application-specific error types for the Tollo IPTV player
#[derive(Debug, thiserror::Error)]
pub enum TolloError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Database initialization failed: {reason}")]
    DatabaseInitialization { reason: String },
    
    #[error("Database migration failed: {reason}")]
    DatabaseMigration { reason: String },
    
    // Network errors
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Failed to fetch playlist from URL: {url}")]
    PlaylistFetch { url: String },
    
    #[error("Failed to download file: {url}")]
    FileDownload { url: String },
    
    // File system errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Failed to create directory: {path}")]
    DirectoryCreation { path: String },
    
    #[error("Failed to access data directory")]
    DataDirectoryAccess,
    
    #[error("Failed to read file: {path}")]
    FileRead { path: String },
    
    #[error("Failed to write file: {path}")]
    FileWrite { path: String },
    
    // Parsing errors
    #[error("Failed to parse M3U playlist: {reason}")]
    M3uParsing { reason: String },
    
    #[error("Failed to parse URL: {url}")]
    UrlParsing { url: String },
    
    #[error("Invalid regex pattern: {pattern}")]
    RegexError { pattern: String },
    
    // Cache errors
    #[error("Cache operation failed: {operation}")]
    Cache { operation: String },
    
    #[error("Image cache error: {reason}")]
    ImageCache { reason: String },
    
    #[error("Search cache error: {reason}")]
    SearchCache { reason: String },
    
    // External process errors
    #[error("Failed to execute external player: {player}")]
    ExternalPlayer { player: String },
    
    #[error("Command execution failed: {command}")]
    CommandExecution { command: String },
    
    // Configuration errors
    #[error("Configuration error: {reason}")]
    Configuration { reason: String },
    
    #[error("Invalid setting value: {key} = {value}")]
    InvalidSetting { key: String, value: String },
    
    // Validation errors
    #[error("Invalid channel ID: {id}")]
    InvalidChannelId { id: String },
    
    #[error("Invalid playlist ID: {id}")]
    InvalidPlaylistId { id: String },
    
    #[error("Invalid URL format: {url}")]
    InvalidUrl { url: String },
    
    // Concurrency errors
    #[error("Failed to acquire lock: {resource}")]
    LockAcquisition { resource: String },
    
    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },
    
    #[error("Operation was cancelled: {operation}")]
    Cancelled { operation: String },
    
    // Application state errors
    #[error("Application not initialized")]
    NotInitialized,
    
    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    // Generic errors
    #[error("Internal error: {reason}")]
    Internal { reason: String },
    
    #[error("Unknown error occurred")]
    Unknown,
}

impl TolloError {
    /// Create a new database initialization error
    pub fn database_init(reason: impl Into<String>) -> Self {
        Self::DatabaseInitialization {
            reason: reason.into(),
        }
    }
    
    /// Create a new directory creation error
    pub fn directory_creation(path: impl Into<String>) -> Self {
        Self::DirectoryCreation {
            path: path.into(),
        }
    }
    
    /// Create a new playlist fetch error
    pub fn playlist_fetch(url: impl Into<String>) -> Self {
        Self::PlaylistFetch {
            url: url.into(),
        }
    }
    
    /// Create a new file download error
    pub fn file_download(url: impl Into<String>) -> Self {
        Self::FileDownload {
            url: url.into(),
        }
    }
    
    /// Create a new M3U parsing error
    pub fn m3u_parsing(reason: impl Into<String>) -> Self {
        Self::M3uParsing {
            reason: reason.into(),
        }
    }
    
    /// Create a new cache error
    pub fn cache(operation: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
        }
    }
    
    /// Create a new lock acquisition error
    pub fn lock_acquisition(resource: impl Into<String>) -> Self {
        Self::LockAcquisition {
            resource: resource.into(),
        }
    }
    
    /// Create a new timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }
    
    /// Create a new internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal {
            reason: reason.into(),
        }
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Network errors are often recoverable
            TolloError::Network(_) | TolloError::PlaylistFetch { .. } | TolloError::FileDownload { .. } => true,
            
            // Timeout and cancellation errors are recoverable
            TolloError::Timeout { .. } | TolloError::Cancelled { .. } => true,
            
            // Cache errors are usually recoverable
            TolloError::Cache { .. } | TolloError::ImageCache { .. } | TolloError::SearchCache { .. } => true,
            
            // Lock acquisition failures might be recoverable
            TolloError::LockAcquisition { .. } => true,
            
            // Most other errors are not recoverable
            _ => false,
        }
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            TolloError::Database(_) => "Database operation failed. Please try again.".to_string(),
            TolloError::DatabaseInitialization { .. } => "Failed to initialize database. Please check your permissions.".to_string(),
            TolloError::Network(_) => "Network connection failed. Please check your internet connection.".to_string(),
            TolloError::PlaylistFetch { .. } => "Failed to load playlist. Please check the URL and try again.".to_string(),
            TolloError::FileDownload { .. } => "Failed to download file. Please check your connection and try again.".to_string(),
            TolloError::DataDirectoryAccess => "Cannot access application data directory. Please check permissions.".to_string(),
            TolloError::M3uParsing { .. } => "Invalid playlist format. Please check the playlist file.".to_string(),
            TolloError::ExternalPlayer { .. } => "Failed to launch media player. Please check player installation.".to_string(),
            TolloError::InvalidUrl { .. } => "Invalid URL format. Please check the URL and try again.".to_string(),
            TolloError::Timeout { .. } => "Operation timed out. Please try again.".to_string(),
            TolloError::NotFound { .. } => "Requested item not found.".to_string(),
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
    
    /// Get error category for logging/telemetry
    pub fn category(&self) -> &'static str {
        match self {
            TolloError::Database(_) | TolloError::DatabaseInitialization { .. } | TolloError::DatabaseMigration { .. } => "database",
            TolloError::Network(_) | TolloError::PlaylistFetch { .. } | TolloError::FileDownload { .. } => "network",
            TolloError::FileSystem(_) | TolloError::DirectoryCreation { .. } | TolloError::DataDirectoryAccess | TolloError::FileRead { .. } | TolloError::FileWrite { .. } => "filesystem",
            TolloError::M3uParsing { .. } | TolloError::UrlParsing { .. } | TolloError::RegexError { .. } => "parsing",
            TolloError::Cache { .. } | TolloError::ImageCache { .. } | TolloError::SearchCache { .. } => "cache",
            TolloError::ExternalPlayer { .. } | TolloError::CommandExecution { .. } => "external",
            TolloError::Configuration { .. } | TolloError::InvalidSetting { .. } => "configuration",
            TolloError::InvalidChannelId { .. } | TolloError::InvalidPlaylistId { .. } | TolloError::InvalidUrl { .. } => "validation",
            TolloError::LockAcquisition { .. } | TolloError::Timeout { .. } | TolloError::Cancelled { .. } => "concurrency",
            TolloError::NotInitialized | TolloError::FeatureNotAvailable { .. } | TolloError::NotFound { .. } => "state",
            TolloError::Internal { .. } | TolloError::Unknown => "internal",
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, TolloError>;

/// Helper trait for converting results to TolloError
pub trait ToTolloError<T> {
    fn to_tollo_error(self) -> Result<T>;
}

impl<T, E> ToTolloError<T> for std::result::Result<T, E>
where
    E: Into<TolloError>,
{
    fn to_tollo_error(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

/// Helper for converting TolloError to String for Tauri commands
impl From<TolloError> for String {
    fn from(error: TolloError) -> String {
        error.user_message()
    }
}

/// Implementation for Tauri InvokeError compatibility
impl From<TolloError> for InvokeError {
    fn from(error: TolloError) -> InvokeError {
        InvokeError::from(error.user_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = TolloError::database_init("Connection failed");
        assert_eq!(
            error.to_string(),
            "Database initialization failed: Connection failed"
        );
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(TolloError::Database(rusqlite::Error::InvalidPath("test".into())).category(), "database");
        assert_eq!(TolloError::PlaylistFetch { url: "test".to_string() }.category(), "network");
        assert_eq!(TolloError::DataDirectoryAccess.category(), "filesystem");
        assert_eq!(TolloError::M3uParsing { reason: "test".to_string() }.category(), "parsing");
        assert_eq!(TolloError::Cache { operation: "test".to_string() }.category(), "cache");
        assert_eq!(TolloError::LockAcquisition { resource: "test".to_string() }.category(), "concurrency");
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(TolloError::PlaylistFetch { url: "test".to_string() }.is_recoverable());
        assert!(TolloError::Timeout { operation: "test".to_string() }.is_recoverable());
        assert!(TolloError::Cache { operation: "test".to_string() }.is_recoverable());
        assert!(TolloError::LockAcquisition { resource: "test".to_string() }.is_recoverable());
        
        assert!(!TolloError::DatabaseInitialization { reason: "test".to_string() }.is_recoverable());
        assert!(!TolloError::DataDirectoryAccess.is_recoverable());
    }

    #[test]
    fn test_user_messages() {
        let db_error = TolloError::Database(rusqlite::Error::InvalidPath("test".into()));
        assert_eq!(db_error.user_message(), "Database operation failed. Please try again.");
        
        let network_error = TolloError::PlaylistFetch { url: "http://example.com".to_string() };
        assert_eq!(network_error.user_message(), "Failed to load playlist. Please check the URL and try again.");
    }

    #[test]
    fn test_error_conversion_to_string() {
        let error = TolloError::timeout("search operation");
        let error_string: String = error.into();
        assert_eq!(error_string, "Operation timed out. Please try again.");
    }

    #[test]
    fn test_builder_methods() {
        let error = TolloError::database_init("Failed to connect");
        assert!(matches!(error, TolloError::DatabaseInitialization { .. }));
        
        let error = TolloError::directory_creation("/tmp/test");
        assert!(matches!(error, TolloError::DirectoryCreation { .. }));
        
        let error = TolloError::playlist_fetch("http://example.com");
        assert!(matches!(error, TolloError::PlaylistFetch { .. }));
    }

    #[test]
    fn test_from_trait_conversions() {
        // Test rusqlite error conversion
        let rusqlite_error = rusqlite::Error::InvalidPath("test".into());
        let tollo_error: TolloError = rusqlite_error.into();
        assert!(matches!(tollo_error, TolloError::Database(_)));
        
        // Test IO error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let tollo_error: TolloError = io_error.into();
        assert!(matches!(tollo_error, TolloError::FileSystem(_)));
    }
}