use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tracing::error;

use crate::app_log;
use crate::database::repository;
use crate::models::*;
use crate::services::processing::{ProcessingService, generate_job_id, create_job};

#[tauri::command]
pub async fn upload_file(
    app: AppHandle,
    path: String,
) -> Result<TranscriptionJob, String> {
    let file_path = PathBuf::from(&path);

    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    let job_id = generate_job_id();
    create_job(job_id.clone());

    app_log!(info, &format!("File upload started: {}", path));

    // Start processing in background
    let job_id_clone = job_id.clone();
    let path_clone = path.clone();

    tokio::spawn(async move {
        let service = match ProcessingService::new() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create processing service: {}", e);
                let _ = app.emit("transcription:error", TranscriptionErrorEvent {
                    job_id: job_id_clone,
                    error: e.to_string(),
                });
                return;
            }
        };

        if let Err(e) = service.process_file(app.clone(), PathBuf::from(path_clone), job_id_clone.clone()).await {
            error!("Processing failed: {}", e);
            let _ = app.emit("transcription:error", TranscriptionErrorEvent {
                job_id: job_id_clone,
                error: e.to_string(),
            });
        }
    });

    Ok(TranscriptionJob {
        job_id,
        file_path: path,
        status: JobStatus::Pending,
    })
}

#[tauri::command]
pub async fn get_transcription(id: i64) -> Result<Transcription, String> {
    repository::get_transcription(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Transcription not found".to_string())
}

#[tauri::command]
pub async fn get_all_transcriptions() -> Result<Vec<TranscriptionSummary>, String> {
    repository::get_all_transcriptions()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_transcriptions(query: String) -> Result<Vec<TranscriptionSummary>, String> {
    repository::search_transcriptions(&query)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_transcription(id: i64) -> Result<(), String> {
    app_log!(info, &format!("Deleting transcription: {}", id));
    repository::delete_transcription(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_transcription(id: i64, format: String) -> Result<String, String> {
    let transcription = repository::get_transcription(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Transcription not found".to_string())?;

    let segments = repository::get_segments(id)
        .map_err(|e| e.to_string())?;

    let speakers = repository::get_speakers(id)
        .map_err(|e| e.to_string())?;

    let content = match format.as_str() {
        "txt" => export_as_txt(&transcription, &segments, &speakers),
        "srt" => export_as_srt(&segments, &speakers),
        "json" => export_as_json(&transcription, &segments, &speakers)?,
        _ => return Err("Unsupported export format".to_string()),
    };

    // Save to file
    let export_dir = repository::get_app_data_dir()
        .map_err(|e| e.to_string())?
        .join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let filename = format!("{}.{}", transcription.title, format);
    let filepath = export_dir.join(&filename);
    std::fs::write(&filepath, content).map_err(|e| e.to_string())?;

    app_log!(info, &format!("Exported transcription to: {:?}", filepath));

    Ok(filepath.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn cancel_processing(job_id: String) -> Result<(), String> {
    use crate::services::processing::ProcessingService;

    if ProcessingService::cancel_job(&job_id) {
        app_log!(info, &format!("Cancelled job: {}", job_id));
        Ok(())
    } else {
        Err("Job not found or already completed".to_string())
    }
}

#[tauri::command]
pub async fn get_processing_status(job_id: String) -> Result<TranscriptionJob, String> {
    use crate::services::processing::ProcessingService;

    let status = ProcessingService::get_job_status(&job_id)
        .ok_or_else(|| "Job not found".to_string())?;

    Ok(TranscriptionJob {
        job_id,
        file_path: String::new(),
        status,
    })
}

fn export_as_txt(
    transcription: &Transcription,
    segments: &[Segment],
    speakers: &[Speaker],
) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {}\n", transcription.title));
    output.push_str(&format!("Date: {}\n", transcription.created_at));
    output.push_str(&format!("Duration: {}s\n", transcription.duration));
    output.push_str(&format!("Language: {}\n\n", transcription.language));

    if segments.is_empty() {
        output.push_str(&transcription.full_text);
    } else {
        for segment in segments {
            let speaker_name = segment.speaker_id
                .and_then(|sid| speakers.iter().find(|s| s.id == sid))
                .map(|s| s.display_name.as_str())
                .unwrap_or("Unknown");

            output.push_str(&format!(
                "[{:.2}] {}: {}\n",
                segment.start_time,
                speaker_name,
                segment.text
            ));
        }
    }

    output
}

fn export_as_srt(segments: &[Segment], speakers: &[Speaker]) -> String {
    let mut output = String::new();

    for (i, segment) in segments.iter().enumerate() {
        let speaker_name = segment.speaker_id
            .and_then(|sid| speakers.iter().find(|s| s.id == sid))
            .map(|s| format!("{}: ", s.display_name))
            .unwrap_or_default();

        output.push_str(&format!("{}\n", i + 1));
        output.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(segment.start_time),
            format_srt_time(segment.end_time)
        ));
        output.push_str(&format!("{}{}\n\n", speaker_name, segment.text));
    }

    output
}

fn format_srt_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millis = ((seconds % 1.0) * 1000.0) as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

fn export_as_json(
    transcription: &Transcription,
    segments: &[Segment],
    speakers: &[Speaker],
) -> Result<String, String> {
    let data = serde_json::json!({
        "transcription": transcription,
        "segments": segments,
        "speakers": speakers,
    });

    serde_json::to_string_pretty(&data)
        .map_err(|e| e.to_string())
}
