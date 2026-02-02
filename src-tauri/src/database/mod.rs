pub mod schema;
pub mod repository;

pub use repository::{
    init_database, get_app_data_dir,
    insert_transcription, get_transcription, get_all_transcriptions,
    search_transcriptions, delete_transcription,
    insert_speaker, get_speakers, update_speaker_name,
    insert_segment, get_segments,
    insert_log, get_logs, clear_logs,
    get_settings, save_settings,
};
