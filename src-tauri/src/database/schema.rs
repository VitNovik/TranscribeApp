use rusqlite::{Connection, Result};
use tracing::info;

pub const SCHEMA_SQL: &str = r#"
-- Transcriptions table
CREATE TABLE IF NOT EXISTS transcriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    duration INTEGER NOT NULL,
    language TEXT NOT NULL,
    model TEXT NOT NULL,
    full_text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Speakers table
CREATE TABLE IF NOT EXISTS speakers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transcription_id INTEGER NOT NULL,
    speaker_label TEXT NOT NULL,
    display_name TEXT NOT NULL,
    color TEXT NOT NULL,
    FOREIGN KEY (transcription_id) REFERENCES transcriptions(id) ON DELETE CASCADE
);

-- Segments table
CREATE TABLE IF NOT EXISTS segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transcription_id INTEGER NOT NULL,
    speaker_id INTEGER,
    text TEXT NOT NULL,
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    confidence REAL,
    FOREIGN KEY (transcription_id) REFERENCES transcriptions(id) ON DELETE CASCADE,
    FOREIGN KEY (speaker_id) REFERENCES speakers(id) ON DELETE SET NULL
);

-- Logs table
CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    context TEXT,
    created_at TEXT NOT NULL
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_segments_transcription ON segments(transcription_id);
CREATE INDEX IF NOT EXISTS idx_segments_speaker ON segments(speaker_id);
CREATE INDEX IF NOT EXISTS idx_speakers_transcription ON speakers(transcription_id);
CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON logs(created_at);
CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at ON transcriptions(created_at);
"#;

pub fn initialize_database(conn: &Connection) -> Result<()> {
    info!("Initializing database schema");

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Execute schema
    conn.execute_batch(SCHEMA_SQL)?;

    info!("Database schema initialized successfully");
    Ok(())
}

pub fn cleanup_old_logs(conn: &Connection, max_entries: i64) -> Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))?;

    if count > max_entries {
        let to_delete = count - max_entries;
        conn.execute(
            "DELETE FROM logs WHERE id IN (SELECT id FROM logs ORDER BY created_at ASC LIMIT ?)",
            [to_delete],
        )
    } else {
        Ok(0)
    }
}
