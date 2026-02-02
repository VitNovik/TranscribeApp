use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub whisper_model: WhisperModel,
    pub default_language: String,
    pub auto_diarization: bool,
    pub manual_speaker_count: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            whisper_model: WhisperModel::Medium,
            default_language: "ru".to_string(),
            auto_diarization: true,
            manual_speaker_count: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
    Medium,
    #[serde(rename = "large-v3")]
    LargeV3,
}

impl WhisperModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WhisperModel::Tiny => "tiny",
            WhisperModel::Base => "base",
            WhisperModel::Small => "small",
            WhisperModel::Medium => "medium",
            WhisperModel::LargeV3 => "large-v3",
        }
    }

    pub fn model_file(&self) -> &'static str {
        match self {
            WhisperModel::Tiny => "ggml-tiny.bin",
            WhisperModel::Base => "ggml-base.bin",
            WhisperModel::Small => "ggml-small.bin",
            WhisperModel::Medium => "ggml-medium.bin",
            WhisperModel::LargeV3 => "ggml-large-v3.bin",
        }
    }
}

impl std::fmt::Display for WhisperModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
