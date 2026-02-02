use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    pub duration: i64,
    pub language: String,
    pub model: String,
    pub full_text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSummary {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub language: String,
    pub speaker_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionJob {
    pub job_id: String,
    pub file_path: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: i64,
    pub transcription_id: i64,
    pub speaker_id: Option<i64>,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTranscription {
    pub title: String,
    pub file_path: String,
    pub duration: i64,
    pub language: String,
    pub model: String,
    pub full_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSegment {
    pub speaker_id: Option<i64>,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessingStage {
    Extracting,
    Transcribing,
    Diarizing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub stage: ProcessingStage,
    pub progress: f64,
    pub eta_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionCompleteEvent {
    pub job_id: String,
    pub transcription_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionErrorEvent {
    pub job_id: String,
    pub error: String,
}
