use std::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};
use tracing::{info, debug, warn, error};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::rolling;

use crate::database::repository::{insert_log, get_app_data_dir};
use crate::models::{LogLevel, LogEntryEvent};

static APP_HANDLE: Lazy<Mutex<Option<AppHandle>>> = Lazy::new(|| Mutex::new(None));

pub fn init_logger() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,transcribe_app=debug"));

    // Setup file appender with daily rotation
    let log_dir = get_app_data_dir()
        .map(|d| d.join("logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/transcribe-app-logs"));

    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = rolling::daily(&log_dir, "app.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so it lives for the duration of the program
    // This is intentional - we want logging to work until process exit
    std::mem::forget(_guard);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_line_number(true))
        .with(
            fmt::layer()
                .with_target(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_writer(file_writer)
        )
        .init();

    info!("Logger initialized");
}

pub fn set_app_handle(handle: AppHandle) {
    let mut app_handle = APP_HANDLE.lock().unwrap();
    *app_handle = Some(handle);
}

/// Log a message and emit to frontend
pub fn log_and_emit(level: LogLevel, message: &str, context: Option<serde_json::Value>) {
    // Log to tracing (goes to both console and file)
    match level {
        LogLevel::Debug => debug!("{}", message),
        LogLevel::Info => info!("{}", message),
        LogLevel::Warning => warn!("{}", message),
        LogLevel::Error => error!("{}", message),
    }

    // Save to database
    let context_str = context.as_ref().map(|c| c.to_string());
    if let Err(e) = insert_log(level.as_str(), message, context_str.as_deref()) {
        error!("Failed to save log to database: {}", e);
    }

    // Emit to frontend
    let event = LogEntryEvent::new(level, message);
    let event = if let Some(ctx) = context {
        event.with_context(ctx)
    } else {
        event
    };

    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        if let Err(e) = handle.emit("log:entry", &event) {
            error!("Failed to emit log event: {}", e);
        }
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! app_log {
    (debug, $msg:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Debug,
            $msg,
            None
        )
    };
    (debug, $msg:expr, $ctx:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Debug,
            $msg,
            Some($ctx)
        )
    };
    (info, $msg:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Info,
            $msg,
            None
        )
    };
    (info, $msg:expr, $ctx:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Info,
            $msg,
            Some($ctx)
        )
    };
    (warn, $msg:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Warning,
            $msg,
            None
        )
    };
    (warn, $msg:expr, $ctx:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Warning,
            $msg,
            Some($ctx)
        )
    };
    (error, $msg:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Error,
            $msg,
            None
        )
    };
    (error, $msg:expr, $ctx:expr) => {
        $crate::services::logger::log_and_emit(
            $crate::models::LogLevel::Error,
            $msg,
            Some($ctx)
        )
    };
}
