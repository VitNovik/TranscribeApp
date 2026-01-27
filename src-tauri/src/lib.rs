//! TranscribeApp - macOS application for audio/video transcription with speaker diarization
//!
//! This is the Tauri backend library that provides:
//! - Audio/video file processing via FFmpeg
//! - Speech-to-text transcription via Whisper
//! - Speaker diarization via pyannote.audio
//! - SQLite database for storing transcriptions
//! - Real-time logging and progress updates

pub mod commands;
pub mod database;
pub mod models;
pub mod services;
pub mod utils;

use tracing::info;

use crate::database::repository::init_database;
use crate::services::logger::{init_logger, set_app_handle};

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging first
    init_logger();

    info!("Starting TranscribeApp");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Set app handle for logger
            set_app_handle(app.handle().clone());

            // Initialize database
            if let Err(e) = init_database() {
                tracing::error!("Failed to initialize database: {}", e);
                return Err(e.into());
            }

            info!("Application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Transcription commands
            commands::upload_file,
            commands::get_transcription,
            commands::get_all_transcriptions,
            commands::search_transcriptions,
            commands::delete_transcription,
            commands::export_transcription,
            commands::cancel_processing,
            commands::get_processing_status,
            // Speaker commands
            commands::get_speakers,
            commands::update_speaker_name,
            commands::get_segments,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            commands::get_available_models,
            commands::download_model,
            // Log commands
            commands::get_logs,
            commands::clear_logs,
            commands::export_logs_to_file,
            commands::copy_logs_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
