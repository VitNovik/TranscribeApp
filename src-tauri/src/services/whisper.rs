use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app_log;
use crate::database::get_app_data_dir;
use crate::models::{WhisperModel, NewSegment};

pub struct WhisperService {
    models_dir: PathBuf,
}

impl WhisperService {
    pub fn new() -> Result<Self> {
        let models_dir = get_app_data_dir()?.join("models");
        std::fs::create_dir_all(&models_dir)?;
        Ok(Self { models_dir })
    }

    /// Get path to model file
    pub fn model_path(&self, model: &WhisperModel) -> PathBuf {
        self.models_dir.join(model.model_file())
    }

    /// Check if model is downloaded
    pub fn is_model_available(&self, model: &WhisperModel) -> bool {
        self.model_path(model).exists()
    }

    /// Get list of available (downloaded) models
    pub fn get_available_models(&self) -> Vec<String> {
        let models = [
            WhisperModel::Tiny,
            WhisperModel::Base,
            WhisperModel::Small,
            WhisperModel::Medium,
            WhisperModel::LargeV3,
        ];

        models
            .iter()
            .filter(|m| self.is_model_available(m))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Download model from Hugging Face
    pub async fn download_model(&self, model: &WhisperModel) -> Result<()> {
        let model_url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
            model.model_file()
        );
        let model_path = self.model_path(model);

        app_log!(info, &format!("Downloading model {} from {}", model.as_str(), model_url));

        // Use curl to download (available on macOS by default)
        let output = Command::new("curl")
            .args([
                "-L",  // Follow redirects
                "-o", model_path.to_str().unwrap(),
                "--progress-bar",
                &model_url,
            ])
            .output()
            .context("Failed to download model")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to download model: {}", stderr);
        }

        app_log!(info, &format!("Model {} downloaded successfully", model.as_str()));
        Ok(())
    }

    /// Transcribe audio file using whisper.cpp (via CLI or library)
    /// For MVP, we'll use a Python whisper implementation via subprocess
    pub fn transcribe(
        &self,
        audio_path: &Path,
        model: &WhisperModel,
        language: &str,
    ) -> Result<TranscriptionResult> {
        app_log!(info, &format!("Whisper: starting transcription with model {} for language {}", model.as_str(), language));

        // For MVP, we'll simulate transcription result
        // In production, this would call whisper.cpp or whisper Python

        // Check if we have a whisper CLI available
        if let Ok(result) = self.transcribe_with_whisper_cli(audio_path, model, language) {
            return Ok(result);
        }

        // Fallback: try Python whisper
        if let Ok(result) = self.transcribe_with_python_whisper(audio_path, model, language) {
            return Ok(result);
        }

        // If no whisper available, return a placeholder result for testing
        app_log!(warn, "No Whisper implementation found, returning placeholder result");
        Ok(TranscriptionResult {
            text: "[Транскрибация недоступна - установите whisper.cpp или whisper Python]".to_string(),
            segments: vec![NewSegment {
                speaker_id: None,
                text: "Транскрибация недоступна. Пожалуйста, установите whisper.cpp или Python whisper.".to_string(),
                start_time: 0.0,
                end_time: 1.0,
                confidence: Some(1.0),
            }],
            language: language.to_string(),
        })
    }

    fn transcribe_with_whisper_cli(
        &self,
        audio_path: &Path,
        model: &WhisperModel,
        language: &str,
    ) -> Result<TranscriptionResult> {
        let model_path = self.model_path(model);

        if !model_path.exists() {
            anyhow::bail!("Model file not found: {:?}", model_path);
        }

        // Try to find whisper.cpp CLI
        let whisper_cmd = which_whisper_cpp()?;

        let output = Command::new(&whisper_cmd)
            .args([
                "-m", model_path.to_str().unwrap(),
                "-l", language,
                "-f", audio_path.to_str().unwrap(),
                "-otxt",  // Output as text
                "-osrt",  // Also output SRT for timestamps
            ])
            .output()
            .context("Failed to execute whisper.cpp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("whisper.cpp failed: {}", stderr);
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse SRT file for segments if available
        let srt_path = audio_path.with_extension("srt");
        let segments = if srt_path.exists() {
            parse_srt_file(&srt_path)?
        } else {
            vec![NewSegment {
                speaker_id: None,
                text: text.clone(),
                start_time: 0.0,
                end_time: 0.0,
                confidence: None,
            }]
        };

        Ok(TranscriptionResult {
            text,
            segments,
            language: language.to_string(),
        })
    }

    fn transcribe_with_python_whisper(
        &self,
        audio_path: &Path,
        model: &WhisperModel,
        language: &str,
    ) -> Result<TranscriptionResult> {
        // Create a Python script for transcription
        let script = format!(r#"
import whisper
import json
import sys

model = whisper.load_model("{}")
result = model.transcribe("{}", language="{}")

output = {{
    "text": result["text"],
    "segments": [
        {{
            "text": s["text"],
            "start": s["start"],
            "end": s["end"]
        }}
        for s in result["segments"]
    ],
    "language": result.get("language", "{}")
}}

print(json.dumps(output))
"#, model.as_str(), audio_path.display(), language, language);

        let output = Command::new("python3")
            .args(["-c", &script])
            .output()
            .context("Failed to execute Python whisper")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Python whisper failed: {}", stderr);
        }

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse whisper output")?;

        let text = result["text"].as_str().unwrap_or("").to_string();
        let segments = result["segments"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|s| NewSegment {
                        speaker_id: None,
                        text: s["text"].as_str().unwrap_or("").to_string(),
                        start_time: s["start"].as_f64().unwrap_or(0.0),
                        end_time: s["end"].as_f64().unwrap_or(0.0),
                        confidence: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(TranscriptionResult {
            text,
            segments,
            language: language.to_string(),
        })
    }
}

impl Default for WhisperService {
    fn default() -> Self {
        Self::new().expect("Failed to create WhisperService")
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<NewSegment>,
    pub language: String,
}

fn which_whisper_cpp() -> Result<PathBuf> {
    // Check common locations
    let paths = [
        "/usr/local/bin/whisper",
        "/opt/homebrew/bin/whisper",
        "whisper",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() || Command::new(path).arg("--help").output().is_ok() {
            return Ok(p);
        }
    }

    anyhow::bail!("whisper.cpp not found")
}

fn parse_srt_file(path: &Path) -> Result<Vec<NewSegment>> {
    let content = std::fs::read_to_string(path)?;
    let mut segments = Vec::new();

    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        // Skip index number
        if line.trim().parse::<u32>().is_ok() {
            // Next line should be timestamps
            if let Some(timestamp_line) = lines.next() {
                let (start, end) = parse_srt_timestamp(timestamp_line)?;

                // Collect text lines until empty line
                let mut text = String::new();
                while let Some(text_line) = lines.peek() {
                    if text_line.trim().is_empty() {
                        lines.next();
                        break;
                    }
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(text_line.trim());
                    lines.next();
                }

                segments.push(NewSegment {
                    speaker_id: None,
                    text,
                    start_time: start,
                    end_time: end,
                    confidence: None,
                });
            }
        }
    }

    Ok(segments)
}

fn parse_srt_timestamp(line: &str) -> Result<(f64, f64)> {
    // Format: 00:00:00,000 --> 00:00:00,000
    let parts: Vec<&str> = line.split(" --> ").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid SRT timestamp: {}", line);
    }

    let start = parse_time(parts[0])?;
    let end = parse_time(parts[1])?;

    Ok((start, end))
}

fn parse_time(s: &str) -> Result<f64> {
    // Format: 00:00:00,000
    let s = s.replace(',', ".");
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid time format: {}", s);
    }

    let hours: f64 = parts[0].parse()?;
    let minutes: f64 = parts[1].parse()?;
    let seconds: f64 = parts[2].parse()?;

    Ok(hours * 3600.0 + minutes * 60.0 + seconds)
}
