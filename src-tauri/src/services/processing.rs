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
        mut cancel_rx: oneshot::Receiver<()>,
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
        Self::update_job_status(&job_id, JobStatus::Processing);

        // Check cancellation helper
        macro_rules! check_cancel {
            ($rx:expr) => {
                if $rx.try_recv().is_ok() {
                    app_log!(info, "Processing cancelled by user");
                    Self::update_job_status(&job_id, JobStatus::Failed);
                    anyhow::bail!("Processing cancelled by user");
                }
            };
        }

        // Step 1: Extract audio
        self.emit_progress(&app, &job_id, ProcessingStage::Extracting, 0.0, 30.0);
        app_log!(info, "Extracting audio from file...");

        let audio_path = self.ffmpeg.extract_audio(&file_path)
            .context("Failed to extract audio")?;

        let duration = self.ffmpeg.get_duration(&audio_path)
            .unwrap_or(0.0);

        self.emit_progress(&app, &job_id, ProcessingStage::Extracting, 100.0, 0.0);
        app_log!(info, &format!("Audio extracted, duration: {:.1}s", duration));

        check_cancel!(cancel_rx);

        // Step 2: Transcribe
        self.emit_progress(&app, &job_id, ProcessingStage::Transcribing, 0.0, duration * 0.5);
        app_log!(info, &format!("Starting transcription with model: {}", model));

        let transcription_result = self.whisper.transcribe(&audio_path, &model, &language)
            .context("Transcription failed")?;

        self.emit_progress(&app, &job_id, ProcessingStage::Transcribing, 100.0, 0.0);
        app_log!(info, &format!("Transcription completed: {} segments", transcription_result.segments.len()));

        check_cancel!(cancel_rx);

        // Step 3: Diarization (if enabled)
        let mut diarization_result = None;

        if settings.auto_diarization {
            self.emit_progress(&app, &job_id, ProcessingStage::Diarizing, 0.0, duration * 0.3);
            app_log!(info, "Starting speaker diarization...");

            match self.diarization.diarize(&audio_path, settings.manual_speaker_count) {
                Ok(result) => {
                    app_log!(info, &format!("Diarization completed: {} speakers, {} segments",
                        result.speakers.len(), result.segments.len()));
                    diarization_result = Some(result);
                }
                Err(e) => {
                    app_log!(warn, &format!("Diarization failed, continuing without speakers: {}", e));
                }
            }
            self.emit_progress(&app, &job_id, ProcessingStage::Diarizing, 100.0, 0.0);
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

        // Save speakers and segments
        if let Some(ref diar) = diarization_result {
            if !diar.speakers.is_empty() {
                // Insert all speakers and collect their DB IDs
                let mut speaker_ids: Vec<i64> = Vec::new();
                for speaker_data in &diar.speakers {
                    let speaker_id = insert_speaker(transcription_id, speaker_data)?;
                    speaker_ids.push(speaker_id);
                }

                // Use diarization service to merge transcription segments with speaker info
                let merged_segments = self.diarization.merge_segments(
                    transcription_result.segments.clone(),
                    diar,
                    &speaker_ids,
                );

                for segment in &merged_segments {
                    insert_segment(transcription_id, segment)?;
                }

                app_log!(info, &format!("Saved {} speakers and {} segments",
                    speaker_ids.len(), merged_segments.len()));
            } else {
                // Diarization returned no speakers - save segments without speaker
                for segment in &transcription_result.segments {
                    insert_segment(transcription_id, segment)?;
                }
            }
        } else {
            // No diarization - save segments without speaker
            for segment in &transcription_result.segments {
                insert_segment(transcription_id, segment)?;
            }
        }

        // Update job status
        Self::update_job_status(&job_id, JobStatus::Completed);

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

    fn update_job_status(job_id: &str, status: JobStatus) {
        if let Ok(mut jobs) = JOBS.lock() {
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = status;
            }
        }
    }

    pub fn cancel_job(job_id: &str) -> bool {
        if let Ok(mut jobs) = JOBS.lock() {
            if let Some(job) = jobs.get_mut(job_id) {
                if let Some(tx) = job.cancel_tx.take() {
                    let _ = tx.send(());
                    job.status = JobStatus::Failed;
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

/// Create a new job and register it, returning a cancel receiver
pub fn create_job(job_id: String) -> oneshot::Receiver<()> {
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let mut jobs = JOBS.lock().unwrap();
    jobs.insert(job_id, JobHandle {
        cancel_tx: Some(cancel_tx),
        status: JobStatus::Pending,
    });
    cancel_rx
}
