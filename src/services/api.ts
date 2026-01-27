import { invoke } from '@tauri-apps/api/core';
import type {
  Transcription,
  TranscriptionSummary,
  TranscriptionJob,
  Speaker,
  Segment,
  Settings,
  LogEntry,
} from '@/types';

// Transcription commands
export async function uploadFile(path: string): Promise<TranscriptionJob> {
  return invoke('upload_file', { path });
}

export async function getTranscription(id: number): Promise<Transcription> {
  return invoke('get_transcription', { id });
}

export async function getAllTranscriptions(): Promise<TranscriptionSummary[]> {
  return invoke('get_all_transcriptions');
}

export async function deleteTranscription(id: number): Promise<void> {
  return invoke('delete_transcription', { id });
}

export async function exportTranscription(id: number, format: string): Promise<string> {
  return invoke('export_transcription', { id, format });
}

export async function searchTranscriptions(query: string): Promise<TranscriptionSummary[]> {
  return invoke('search_transcriptions', { query });
}

// Speaker commands
export async function getSpeakers(transcriptionId: number): Promise<Speaker[]> {
  return invoke('get_speakers', { transcriptionId });
}

export async function updateSpeakerName(speakerId: number, newName: string): Promise<void> {
  return invoke('update_speaker_name', { speakerId, newName });
}

export async function getSegments(transcriptionId: number): Promise<Segment[]> {
  return invoke('get_segments', { transcriptionId });
}

// Settings commands
export async function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke('update_settings', { settings });
}

export async function getAvailableModels(): Promise<string[]> {
  return invoke('get_available_models');
}

export async function downloadModel(modelName: string): Promise<void> {
  return invoke('download_model', { modelName });
}

// Log commands
export async function getLogs(levelFilter?: string, limit?: number): Promise<LogEntry[]> {
  return invoke('get_logs', { levelFilter, limit });
}

export async function clearLogs(): Promise<void> {
  return invoke('clear_logs');
}

export async function exportLogsToFile(): Promise<string> {
  return invoke('export_logs_to_file');
}

export async function copyLogsToClipboard(): Promise<void> {
  return invoke('copy_logs_to_clipboard');
}

// Utility commands
export async function cancelProcessing(jobId: string): Promise<void> {
  return invoke('cancel_processing', { jobId });
}

export async function getProcessingStatus(jobId: string): Promise<TranscriptionJob> {
  return invoke('get_processing_status', { jobId });
}

// File dialog
export async function selectFile(): Promise<string | null> {
  return invoke('select_file');
}
