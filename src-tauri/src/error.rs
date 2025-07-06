use tauri::ipc::InvokeError;

/// Application-specific error types for the Töllö IPTV player
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