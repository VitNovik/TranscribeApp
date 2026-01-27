use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tracing::{debug, info, error};

use crate::models::*;
use super::schema::{initialize_database, cleanup_old_logs};

const MAX_LOG_ENTRIES: i64 = 1000;

pub static DATABASE: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

pub fn get_app_data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let data_dir = home.join(".transcribe");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

pub fn init_database() -> Result<()> {
    let data_dir = get_app_data_dir()?;
    let db_path = data_dir.join("data.db");

    info!("Opening database at {:?}", db_path);

    let conn = Connection::open(&db_path)?;
    initialize_database(&conn)?;

    let mut db = DATABASE.lock().unwrap();
    *db = Some(conn);

    info!("Database initialized successfully");
    Ok(())
}

fn with_connection<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&Connection) -> Result<T>,
{
    let db = DATABASE.lock().unwrap();
    let conn = db.as_ref().context("Database not initialized")?;
    f(conn)
}

// Transcription operations
pub fn insert_transcription(new: &NewTranscription) -> Result<i64> {
    with_connection(|conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO transcriptions (title, file_path, duration, language, model, full_text, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![new.title, new.file_path, new.duration, new.language, new.model, new.full_text, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn get_transcription(id: i64) -> Result<Option<Transcription>> {
    with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, duration, language, model, full_text, created_at, updated_at
             FROM transcriptions WHERE id = ?1"
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(Transcription {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                duration: row.get(3)?,
                language: row.get(4)?,
                model: row.get(5)?,
                full_text: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        });

        match result {
            Ok(t) => Ok(Some(t)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    })
}

pub fn get_all_transcriptions() -> Result<Vec<TranscriptionSummary>> {
    with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.title, t.duration, t.language,
                    (SELECT COUNT(*) FROM speakers s WHERE s.transcription_id = t.id) as speaker_count,
                    t.created_at
             FROM transcriptions t
             ORDER BY t.created_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TranscriptionSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                duration: row.get(2)?,
                language: row.get(3)?,
                speaker_count: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

pub fn search_transcriptions(query: &str) -> Result<Vec<TranscriptionSummary>> {
    with_connection(|conn| {
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT t.id, t.title, t.duration, t.language,
                    (SELECT COUNT(*) FROM speakers s WHERE s.transcription_id = t.id) as speaker_count,
                    t.created_at
             FROM transcriptions t
             WHERE t.title LIKE ?1 OR t.full_text LIKE ?1
             ORDER BY t.created_at DESC"
        )?;

        let rows = stmt.query_map([&search_pattern], |row| {
            Ok(TranscriptionSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                duration: row.get(2)?,
                language: row.get(3)?,
                speaker_count: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

pub fn delete_transcription(id: i64) -> Result<()> {
    with_connection(|conn| {
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", [id])?;
        Ok(())
    })
}

// Speaker operations
pub fn insert_speaker(transcription_id: i64, new: &NewSpeaker) -> Result<i64> {
    with_connection(|conn| {
        conn.execute(
            "INSERT INTO speakers (transcription_id, speaker_label, display_name, color)
             VALUES (?1, ?2, ?3, ?4)",
            params![transcription_id, new.speaker_label, new.display_name, new.color],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn get_speakers(transcription_id: i64) -> Result<Vec<Speaker>> {
    with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, transcription_id, speaker_label, display_name, color
             FROM speakers WHERE transcription_id = ?1
             ORDER BY speaker_label"
        )?;

        let rows = stmt.query_map([transcription_id], |row| {
            Ok(Speaker {
                id: row.get(0)?,
                transcription_id: row.get(1)?,
                speaker_label: row.get(2)?,
                display_name: row.get(3)?,
                color: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

pub fn update_speaker_name(speaker_id: i64, new_name: &str) -> Result<()> {
    with_connection(|conn| {
        conn.execute(
            "UPDATE speakers SET display_name = ?1 WHERE id = ?2",
            params![new_name, speaker_id],
        )?;
        Ok(())
    })
}

// Segment operations
pub fn insert_segment(transcription_id: i64, new: &NewSegment) -> Result<i64> {
    with_connection(|conn| {
        conn.execute(
            "INSERT INTO segments (transcription_id, speaker_id, text, start_time, end_time, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![transcription_id, new.speaker_id, new.text, new.start_time, new.end_time, new.confidence],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn get_segments(transcription_id: i64) -> Result<Vec<Segment>> {
    with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, transcription_id, speaker_id, text, start_time, end_time, confidence
             FROM segments WHERE transcription_id = ?1
             ORDER BY start_time"
        )?;

        let rows = stmt.query_map([transcription_id], |row| {
            Ok(Segment {
                id: row.get(0)?,
                transcription_id: row.get(1)?,
                speaker_id: row.get(2)?,
                text: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                confidence: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

// Log operations
pub fn insert_log(level: &str, message: &str, context: Option<&str>) -> Result<i64> {
    with_connection(|conn| {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        conn.execute(
            "INSERT INTO logs (level, message, context, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![level, message, context, now],
        )?;

        // Cleanup old logs
        if let Err(e) = cleanup_old_logs(conn, MAX_LOG_ENTRIES) {
            error!("Failed to cleanup old logs: {}", e);
        }

        Ok(conn.last_insert_rowid())
    })
}

pub fn get_logs(level_filter: Option<&str>, limit: usize) -> Result<Vec<LogEntry>> {
    with_connection(|conn| {
        let mut results = Vec::new();

        let parse_log_entry = |row: &rusqlite::Row| -> rusqlite::Result<LogEntry> {
            Ok(LogEntry {
                id: row.get(0)?,
                level: match row.get::<_, String>(1)?.as_str() {
                    "DEBUG" => LogLevel::Debug,
                    "INFO" => LogLevel::Info,
                    "WARNING" => LogLevel::Warning,
                    "ERROR" => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                message: row.get(2)?,
                context: row.get(3)?,
                created_at: row.get(4)?,
            })
        };

        match level_filter {
            Some(level) => {
                let mut stmt = conn.prepare(
                    "SELECT id, level, message, context, created_at FROM logs
                     WHERE level = ?1 ORDER BY created_at DESC LIMIT ?2"
                )?;
                let rows = stmt.query_map(params![level, limit as i64], parse_log_entry)?;
                for row in rows.flatten() {
                    results.push(row);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, level, message, context, created_at FROM logs
                     ORDER BY created_at DESC LIMIT ?1"
                )?;
                let rows = stmt.query_map(params![limit as i64], parse_log_entry)?;
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        };

        results.reverse(); // Return in chronological order
        Ok(results)
    })
}

pub fn clear_logs() -> Result<()> {
    with_connection(|conn| {
        conn.execute("DELETE FROM logs", [])?;
        Ok(())
    })
}

// Settings operations
pub fn get_settings() -> Result<Settings> {
    with_connection(|conn| {
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut settings = Settings::default();

        for row in rows.flatten() {
            match row.0.as_str() {
                "whisper_model" => {
                    settings.whisper_model = serde_json::from_str(&row.1).unwrap_or(WhisperModel::Medium);
                }
                "default_language" => {
                    settings.default_language = row.1;
                }
                "auto_diarization" => {
                    settings.auto_diarization = row.1.parse().unwrap_or(true);
                }
                "manual_speaker_count" => {
                    settings.manual_speaker_count = row.1.parse().ok();
                }
                _ => {}
            }
        }

        Ok(settings)
    })
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    with_connection(|conn| {
        let pairs = [
            ("whisper_model", serde_json::to_string(&settings.whisper_model)?),
            ("default_language", settings.default_language.clone()),
            ("auto_diarization", settings.auto_diarization.to_string()),
            ("manual_speaker_count", settings.manual_speaker_count.map(|n| n.to_string()).unwrap_or_default()),
        ];

        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }

        debug!("Settings saved successfully");
        Ok(())
    })
}
