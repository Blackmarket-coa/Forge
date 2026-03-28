use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ForgeError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ConfigNotFound(String),
    ConfigInvalid(String),
    ProjectNotFound(String),
    ProcessError(String),
    TauriCliNotFound,
}

impl Display for ForgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::ConfigNotFound(path) => write!(f, "Config not found: {path}"),
            Self::ConfigInvalid(msg) => write!(f, "Config invalid: {msg}"),
            Self::ProjectNotFound(id) => write!(f, "Project not found: {id}"),
            Self::ProcessError(msg) => write!(f, "Process error: {msg}"),
            Self::TauriCliNotFound => write!(f, "Tauri CLI not found"),
        }
    }
}

impl Error for ForgeError {}

impl From<std::io::Error> for ForgeError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ForgeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<ForgeError> for String {
    fn from(value: ForgeError) -> Self {
        value.to_string()
    }
}
