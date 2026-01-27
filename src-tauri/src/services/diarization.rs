use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

use crate::models::{NewSpeaker, NewSegment};
use crate::models::speaker::{get_speaker_color, generate_speaker_name};

pub struct DiarizationService;

impl DiarizationService {
    pub fn new() -> Self {
        Self
    }

    /// Run speaker diarization using pyannote.audio
    pub fn diarize(
        &self,
        audio_path: &Path,
        num_speakers: Option<i32>,
    ) -> Result<DiarizationResult> {
        info!("Starting diarization for {:?}", audio_path);

        // Try to run pyannote diarization via Python
        match self.run_pyannote_diarization(audio_path, num_speakers) {
            Ok(result) => {
                info!("Diarization completed: {} speakers found", result.speakers.len());
                return Ok(result);
            }
            Err(e) => {
                warn!("Pyannote diarization failed: {}, falling back to single speaker", e);
            }
        }

        // Fallback: return single speaker result
        Ok(DiarizationResult {
            speakers: vec![NewSpeaker {
                speaker_label: "SPEAKER_00".to_string(),
                display_name: generate_speaker_name(0),
                color: get_speaker_color(0).to_string(),
            }],
            segments: vec![],
        })
    }

    fn run_pyannote_diarization(
        &self,
        audio_path: &Path,
        num_speakers: Option<i32>,
    ) -> Result<DiarizationResult> {
        let num_speakers_arg = num_speakers
            .map(|n| format!(", num_speakers={}", n))
            .unwrap_or_default();

        let script = format!(r#"
import json
from pyannote.audio import Pipeline

pipeline = Pipeline.from_pretrained(
    "pyannote/speaker-diarization-3.1",
    use_auth_token=True  # Requires HF token
)

diarization = pipeline("{audio_path}"{num_speakers_arg})

speakers = set()
segments = []

for turn, _, speaker in diarization.itertracks(yield_label=True):
    speakers.add(speaker)
    segments.append({{
        "speaker": speaker,
        "start": turn.start,
        "end": turn.end
    }})

speaker_list = sorted(list(speakers))
speaker_map = {{s: i for i, s in enumerate(speaker_list)}}

result = {{
    "speakers": [
        {{
            "label": s,
            "index": speaker_map[s]
        }}
        for s in speaker_list
    ],
    "segments": [
        {{
            "speaker_index": speaker_map[s["speaker"]],
            "start": s["start"],
            "end": s["end"]
        }}
        for s in segments
    ]
}}

print(json.dumps(result))
"#, audio_path = audio_path.display(), num_speakers_arg = num_speakers_arg);

        let output = Command::new("python3")
            .args(["-c", &script])
            .output()
            .context("Failed to execute pyannote diarization")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Pyannote diarization failed: {}", stderr);
        }

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse diarization output")?;

        let speakers: Vec<NewSpeaker> = result["speakers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|s| {
                        let index = s["index"].as_u64().unwrap_or(0) as usize;
                        NewSpeaker {
                            speaker_label: s["label"].as_str().unwrap_or("SPEAKER_00").to_string(),
                            display_name: generate_speaker_name(index),
                            color: get_speaker_color(index).to_string(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let segments: Vec<DiarizationSegment> = result["segments"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|s| DiarizationSegment {
                        speaker_index: s["speaker_index"].as_u64().unwrap_or(0) as usize,
                        start: s["start"].as_f64().unwrap_or(0.0),
                        end: s["end"].as_f64().unwrap_or(0.0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(DiarizationResult { speakers, segments })
    }

    /// Merge transcription segments with diarization results
    pub fn merge_segments(
        &self,
        transcription_segments: Vec<NewSegment>,
        diarization: &DiarizationResult,
        speaker_ids: &[i64],
    ) -> Vec<NewSegment> {
        if diarization.segments.is_empty() {
            return transcription_segments;
        }

        transcription_segments
            .into_iter()
            .map(|mut segment| {
                // Find the diarization segment that overlaps most with this transcription segment
                let midpoint = (segment.start_time + segment.end_time) / 2.0;

                if let Some(diar_seg) = diarization.segments.iter().find(|d| {
                    d.start <= midpoint && midpoint <= d.end
                }) {
                    if diar_seg.speaker_index < speaker_ids.len() {
                        segment.speaker_id = Some(speaker_ids[diar_seg.speaker_index]);
                    }
                }

                segment
            })
            .collect()
    }

    /// Check if pyannote is available
    pub fn is_available() -> bool {
        let output = Command::new("python3")
            .args(["-c", "import pyannote.audio; print('ok')"])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }
}

impl Default for DiarizationService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DiarizationResult {
    pub speakers: Vec<NewSpeaker>,
    pub segments: Vec<DiarizationSegment>,
}

#[derive(Debug, Clone)]
pub struct DiarizationSegment {
    pub speaker_index: usize,
    pub start: f64,
    pub end: f64,
}
