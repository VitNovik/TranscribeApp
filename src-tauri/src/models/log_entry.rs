use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub level: LogLevel,
    pub message: String,
    pub context: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(level: tracing::Level) -> Self {
        if level == tracing::Level::ERROR {
            LogLevel::Error
        } else if level == tracing::Level::WARN {
            LogLevel::Warning
        } else if level == tracing::Level::INFO {
            LogLevel::Info
        } else {
            LogLevel::Debug
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryEvent {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
    pub context: Option<serde_json::Value>,
}

impl LogEntryEvent {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            context: None,
        }
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}
