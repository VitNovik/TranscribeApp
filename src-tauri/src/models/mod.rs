pub mod transcription;
pub mod speaker;
pub mod settings;
pub mod log_entry;

// Transcription types
pub use transcription::{
    Transcription, TranscriptionSummary, TranscriptionJob,
    JobStatus, Segment, NewTranscription, NewSegment,
    ProcessingStage, ProgressEvent,
    TranscriptionCompleteEvent, TranscriptionErrorEvent,
};

// Speaker types
pub use speaker::{Speaker, NewSpeaker, SPEAKER_COLORS};

// Settings types
pub use settings::{Settings, WhisperModel};

// Log types
pub use log_entry::{LogEntry, LogLevel, LogEntryEvent};
