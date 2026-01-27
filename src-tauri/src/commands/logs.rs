use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri::AppHandle;

use crate::database::repository;
use crate::models::*;

#[tauri::command]
pub async fn get_logs(
    level_filter: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    let limit = limit.unwrap_or(1000);
    repository::get_logs(level_filter.as_deref(), limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    repository::clear_logs()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_logs_to_file() -> Result<String, String> {
    let logs = repository::get_logs(None, 10000)
        .map_err(|e| e.to_string())?;

    let content = logs
        .iter()
        .map(|log| {
            format!(
                "[{}] {} {}",
                log.created_at,
                log.level.as_str(),
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let export_dir = repository::get_app_data_dir()
        .map_err(|e| e.to_string())?
        .join("logs");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let filename = format!(
        "transcribe-app-logs-{}.txt",
        chrono::Local::now().format("%Y-%m-%d-%H%M%S")
    );
    let filepath = export_dir.join(&filename);

    std::fs::write(&filepath, content).map_err(|e| e.to_string())?;

    Ok(filepath.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn copy_logs_to_clipboard(app: AppHandle) -> Result<(), String> {
    let logs = repository::get_logs(None, 10000)
        .map_err(|e| e.to_string())?;

    let content = logs
        .iter()
        .map(|log| {
            format!(
                "[{}] {} {}",
                log.created_at,
                log.level.as_str(),
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    app.clipboard()
        .write_text(content)
        .map_err(|e| e.to_string())
}
