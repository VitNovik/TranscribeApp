use crate::app_log;
use crate::database::repository;
use crate::models::{Settings, WhisperModel};
use crate::services::WhisperService;

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    repository::get_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(settings: Settings) -> Result<(), String> {
    app_log!(info, &format!("Updating settings: model={}", settings.whisper_model));
    repository::save_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_models() -> Result<Vec<String>, String> {
    let service = WhisperService::new()
        .map_err(|e| e.to_string())?;

    Ok(service.get_available_models())
}

#[tauri::command]
pub async fn download_model(model_name: String) -> Result<(), String> {
    let service = WhisperService::new()
        .map_err(|e| e.to_string())?;

    let model = match model_name.as_str() {
        "tiny" => WhisperModel::Tiny,
        "base" => WhisperModel::Base,
        "small" => WhisperModel::Small,
        "medium" => WhisperModel::Medium,
        "large-v3" => WhisperModel::LargeV3,
        _ => return Err("Invalid model name".to_string()),
    };

    app_log!(info, &format!("Downloading model: {}", model_name));

    service.download_model(&model).await
        .map_err(|e| e.to_string())
}
