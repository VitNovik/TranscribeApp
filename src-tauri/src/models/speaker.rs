use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub id: i64,
    pub transcription_id: i64,
    pub speaker_label: String,
    pub display_name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSpeaker {
    pub speaker_label: String,
    pub display_name: String,
    pub color: String,
}

pub const SPEAKER_COLORS: [&str; 8] = [
    "#3B82F6", // blue
    "#10B981", // green
    "#F59E0B", // yellow
    "#EF4444", // red
    "#8B5CF6", // purple
    "#EC4899", // pink
    "#06B6D4", // cyan
    "#F97316", // orange
];

pub fn get_speaker_color(index: usize) -> &'static str {
    SPEAKER_COLORS[index % SPEAKER_COLORS.len()]
}

pub fn generate_speaker_name(index: usize) -> String {
    format!("Спикер {}", index + 1)
}
