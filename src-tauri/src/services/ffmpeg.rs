use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app_log;
use crate::database::get_app_data_dir;
use crate::utils::path_to_str;

pub struct FFmpegService;

impl FFmpegService {
    pub fn new() -> Self {
        Self
    }

    /// Extract audio from video/audio file to WAV format (16kHz mono)
    pub fn extract_audio(&self, input_path: &Path) -> Result<PathBuf> {
        let temp_dir = get_app_data_dir()?.join("temp");
        std::fs::create_dir_all(&temp_dir)?;

        let output_filename = format!("{}.wav", uuid::Uuid::new_v4());
        let output_path = temp_dir.join(&output_filename);
        let input_str = path_to_str(input_path, "Input file")?;
        let output_str = path_to_str(&output_path, "Output file")?;

        app_log!(
            info,
            &format!("FFmpeg: extracting audio from {:?}", input_path)
        );

        let ffmpeg_args = [
            "-i",
            input_str,
            "-vn", // No video
            "-acodec",
            "pcm_s16le", // 16-bit PCM
            "-ar",
            "16000", // 16kHz sample rate (required by Whisper)
            "-ac",
            "1",  // Mono
            "-y", // Overwrite output
            output_str,
        ];

        app_log!(
            debug,
            &format!("FFmpeg command: ffmpeg {}", ffmpeg_args.join(" "))
        );

        let output = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .output()
            .context("Failed to execute FFmpeg. Is FFmpeg installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            app_log!(error, &format!("FFmpeg error: {}", stderr));
            anyhow::bail!("FFmpeg failed: {}", stderr);
        }

        app_log!(debug, "FFmpeg: audio extraction completed successfully");
        Ok(output_path)
    }

    /// Get duration of media file in seconds
    pub fn get_duration(&self, file_path: &Path) -> Result<f64> {
        let file_str = path_to_str(file_path, "Media file")?;
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                file_str,
            ])
            .output()
            .context("Failed to execute FFprobe")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FFprobe failed: {}", stderr);
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration: f64 = duration_str
            .trim()
            .parse()
            .context("Failed to parse duration")?;

        app_log!(
            debug,
            &format!("FFprobe: media duration = {:.1}s", duration)
        );
        Ok(duration)
    }

    /// Check if FFmpeg is available
    pub fn is_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Cleanup temporary files
    pub fn cleanup_temp_file(&self, path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
            app_log!(debug, &format!("Cleaned up temp file: {:?}", path));
        }
        Ok(())
    }
}

impl Default for FFmpegService {
    fn default() -> Self {
        Self::new()
    }
}

/// Get file extension (lowercase)
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Check if file is an audio file
pub fn is_audio_file(path: &Path) -> bool {
    matches!(
        get_extension(path).as_deref(),
        Some("mp3" | "wav" | "m4a" | "flac" | "ogg")
    )
}

/// Check if file is a video file
pub fn is_video_file(path: &Path) -> bool {
    matches!(
        get_extension(path).as_deref(),
        Some("mp4" | "mov" | "avi" | "mkv" | "webm")
    )
}

/// Check if file is supported
pub fn is_supported_file(path: &Path) -> bool {
    is_audio_file(path) || is_video_file(path)
}
