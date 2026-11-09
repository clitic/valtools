use std::fmt;

/// Unified error type for the application.
#[derive(Debug)]
pub enum AppError {
    /// I/O errors (file read/write, process spawning).
    Io(std::io::Error),
    /// YAML parsing errors.
    Yaml(String),
    /// JSON parsing errors.
    Json(String),
    /// HTTP request failures.
    Http(String),
    /// Riot API / auth errors (expired session, missing token, etc.).
    Api(String),
    /// General application logic errors.
    App(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Yaml(e) => write!(f, "YAML parse error: {}", e),
            Self::Json(e) => write!(f, "JSON parse error: {}", e),
            Self::Http(e) => write!(f, "Request failed: {}", e),
            Self::Api(e) => write!(f, "{}", e),
            Self::App(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Yaml(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

// Allow converting from &str for quick error construction.
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::App(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::App(s)
    }
}
