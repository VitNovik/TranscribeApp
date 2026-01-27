use crate::app_log;
use crate::database::repository;
use crate::models::*;

#[tauri::command]
pub async fn get_speakers(transcription_id: i64) -> Result<Vec<Speaker>, String> {
    repository::get_speakers(transcription_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_speaker_name(speaker_id: i64, new_name: String) -> Result<(), String> {
    app_log!(info, &format!("Updating speaker {} name to: {}", speaker_id, new_name));
    repository::update_speaker_name(speaker_id, &new_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_segments(transcription_id: i64) -> Result<Vec<Segment>, String> {
    repository::get_segments(transcription_id)
        .map_err(|e| e.to_string())
}
