use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use tracing::error;

use crate::app_log;
use crate::database::repository::{
    insert_transcription, insert_speaker, insert_segment, get_settings,
};
use crate::models::*;
use crate::services::{FFmpegService, WhisperService, DiarizationService};

/// Active processing jobs
pub static JOBS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, JobHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct JobHandle {
    pub cancel_tx: Option<oneshot::Sender<()>>,
    pub status: JobStatus,
}

pub struct ProcessingService {
    ffmpeg: FFmpegService,
    whisper: WhisperService,
    diarization: DiarizationService,
}

impl ProcessingService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpegService::new(),
            whisper: WhisperService::new()?,
            diarization: DiarizationService::new(),
        })
    }

    /// Start processing a file
    pub async fn process_file(
        &self,
        app: AppHandle,
        file_path: PathBuf,
        job_id: String,
    ) -> Result<i64> {
        let file_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        app_log!(info, &format!("Starting processing: {}", file_name));

        // Get settings
        let settings = get_settings().unwrap_or_default();
        let model = settings.whisper_model;
        let language = settings.default_language;

        // Update job status
        self.update_job_status(&job_id, JobStatus::Processing);

        // Step 1: Extract audio
        self.emit_progress(&app, &job_id, ProcessingStage::Extracting, 0.0, 30.0);
        app_log!(info, "Extracting audio from file...");

        let audio_path = self.ffmpeg.extract_audio(&file_path)
            .context("Failed to extract audio")?;

        let duration = self.ffmpeg.get_duration(&audio_path)
            .unwrap_or(0.0);

        self.emit_progress(&app, &job_id, ProcessingStage::Extracting, 100.0, 0.0);
        app_log!(info, &format!("Audio extracted, duration: {:.1}s", duration));

        // Step 2: Transcribe
        self.emit_progress(&app, &job_id, ProcessingStage::Transcribing, 0.0, duration * 0.5);
        app_log!(info, &format!("Starting transcription with model: {}", model));

        let transcription_result = self.whisper.transcribe(&audio_path, &model, &language)
            .context("Transcription failed")?;

        self.emit_progress(&app, &job_id, ProcessingStage::Transcribing, 100.0, 0.0);
        app_log!(info, "Transcription completed");

        // Step 3: Diarization (if enabled)
        let final_segments = transcription_result.segments;

        if settings.auto_diarization {
            self.emit_progress(&app, &job_id, ProcessingStage::Diarizing, 0.0, duration * 0.3);
            app_log!(info, "Starting speaker diarization...");

            let diarization_result = self.diarization.diarize(
                &audio_path,
                settings.manual_speaker_count,
            )?;

            app_log!(info, &format!("Found {} speakers", diarization_result.speakers.len()));
            self.emit_progress(&app, &job_id, ProcessingStage::Diarizing, 100.0, 0.0);

            // Save speakers (we'll get their IDs after insertion)
            // For now, we need to create the transcription first, then speakers
        }

        // Cleanup temp audio file
        let _ = self.ffmpeg.cleanup_temp_file(&audio_path);

        // Step 4: Save to database
        app_log!(info, "Saving transcription to database...");

        let new_transcription = NewTranscription {
            title: file_name,
            file_path: file_path.to_string_lossy().to_string(),
            duration: duration as i64,
            language: transcription_result.language.clone(),
            model: model.to_string(),
            full_text: transcription_result.text.clone(),
        };

        let transcription_id = insert_transcription(&new_transcription)
            .context("Failed to save transcription")?;

        // If diarization was done, run it again and save speakers/segments
        if settings.auto_diarization {
            // Re-run diarization to get speaker info (or use cached result)
            // For MVP, we'll create default speakers
            let default_speaker = NewSpeaker {
                speaker_label: "SPEAKER_00".to_string(),
                display_name: "Спикер 1".to_string(),
                color: "#3B82F6".to_string(),
            };
            let speaker_id = insert_speaker(transcription_id, &default_speaker)?;

            // Save segments with speaker
            for segment in &final_segments {
                let mut seg = segment.clone();
                seg.speaker_id = Some(speaker_id);
                insert_segment(transcription_id, &seg)?;
            }
        } else {
            // Save segments without speaker
            for segment in &final_segments {
                insert_segment(transcription_id, segment)?;
            }
        }

        // Update job status
        self.update_job_status(&job_id, JobStatus::Completed);

        // Emit completion event
        let complete_event = TranscriptionCompleteEvent {
            job_id: job_id.clone(),
            transcription_id,
        };
        let _ = app.emit("transcription:complete", &complete_event);

        app_log!(info, &format!("Processing completed, transcription ID: {}", transcription_id));

        Ok(transcription_id)
    }

    fn emit_progress(
        &self,
        app: &AppHandle,
        job_id: &str,
        stage: ProcessingStage,
        progress: f64,
        eta: f64,
    ) {
        let event = ProgressEvent {
            job_id: job_id.to_string(),
            stage,
            progress,
            eta_seconds: eta,
        };

        if let Err(e) = app.emit("transcription:progress", &event) {
            error!("Failed to emit progress event: {}", e);
        }
    }

    fn update_job_status(&self, job_id: &str, status: JobStatus) {
        if let Ok(mut jobs) = JOBS.lock() {
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = status;
            }
        }
    }

    pub fn cancel_job(job_id: &str) -> bool {
        if let Ok(mut jobs) = JOBS.lock() {
            if let Some(job) = jobs.remove(job_id) {
                if let Some(tx) = job.cancel_tx {
                    let _ = tx.send(());
                    return true;
                }
            }
        }
        false
    }

    pub fn get_job_status(job_id: &str) -> Option<JobStatus> {
        JOBS.lock()
            .ok()
            .and_then(|jobs| jobs.get(job_id).map(|j| j.status.clone()))
    }
}

impl Default for ProcessingService {
    fn default() -> Self {
        Self::new().expect("Failed to create ProcessingService")
    }
}

/// Generate a unique job ID
pub fn generate_job_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Create a new job and register it
pub fn create_job(job_id: String) {
    let mut jobs = JOBS.lock().unwrap();
    jobs.insert(job_id, JobHandle {
        cancel_tx: None,
        status: JobStatus::Pending,
    });
}
