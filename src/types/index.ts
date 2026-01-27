// Transcription types
export interface Transcription {
  id: number;
  title: string;
  file_path: string;
  duration: number;
  language: string;
  model: string;
  full_text: string;
  created_at: string;
  updated_at: string;
}

export interface TranscriptionSummary {
  id: number;
  title: string;
  duration: number;
  language: string;
  speaker_count: number;
  created_at: string;
}

export interface TranscriptionJob {
  job_id: string;
  file_path: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
}

// Speaker types
export interface Speaker {
  id: number;
  transcription_id: number;
  speaker_label: string;
  display_name: string;
  color: string;
}

// Segment types
export interface Segment {
  id: number;
  transcription_id: number;
  speaker_id: number | null;
  text: string;
  start_time: number;
  end_time: number;
  confidence: number | null;
}

export interface SegmentWithSpeaker extends Segment {
  speaker?: Speaker;
}

// Log types
export type LogLevel = 'DEBUG' | 'INFO' | 'WARNING' | 'ERROR';

export interface LogEntry {
  id: number;
  level: LogLevel;
  message: string;
  context: string | null;
  created_at: string;
}

// Settings types
export type WhisperModel = 'tiny' | 'base' | 'small' | 'medium' | 'large-v3';

export interface Settings {
  whisper_model: WhisperModel;
  default_language: string;
  auto_diarization: boolean;
  manual_speaker_count: number | null;
}

// Progress event types
export type ProcessingStage = 'extracting' | 'transcribing' | 'diarizing';

export interface ProgressEvent {
  job_id: string;
  stage: ProcessingStage;
  progress: number;
  eta_seconds: number;
}

export interface TranscriptionCompleteEvent {
  job_id: string;
  transcription_id: number;
}

export interface TranscriptionErrorEvent {
  job_id: string;
  error: string;
}

export interface LogEntryEvent {
  level: LogLevel;
  message: string;
  timestamp: string;
  context?: Record<string, unknown>;
}

// UI State types
export interface ProcessingState {
  isProcessing: boolean;
  currentJobId: string | null;
  stage: ProcessingStage | null;
  progress: number;
  eta: number;
}

// Speaker colors for UI
export const SPEAKER_COLORS = [
  '#3B82F6', // blue
  '#10B981', // green
  '#F59E0B', // yellow
  '#EF4444', // red
  '#8B5CF6', // purple
  '#EC4899', // pink
  '#06B6D4', // cyan
  '#F97316', // orange
] as const;

// Supported file formats
export const AUDIO_FORMATS = ['.mp3', '.wav', '.m4a', '.flac', '.ogg'] as const;
export const VIDEO_FORMATS = ['.mp4', '.mov', '.avi', '.mkv', '.webm'] as const;
export const SUPPORTED_FORMATS = [...AUDIO_FORMATS, ...VIDEO_FORMATS] as const;

// Languages
export const SUPPORTED_LANGUAGES = [
  { code: 'ru', name: 'Русский' },
  { code: 'en', name: 'English' },
  { code: 'de', name: 'Deutsch' },
  { code: 'fr', name: 'Français' },
  { code: 'es', name: 'Español' },
  { code: 'it', name: 'Italiano' },
  { code: 'pt', name: 'Português' },
  { code: 'zh', name: '中文' },
  { code: 'ja', name: '日本語' },
  { code: 'ko', name: '한국어' },
] as const;
