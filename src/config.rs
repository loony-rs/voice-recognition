//! Get default config

use crate::realtime::models::{self, AudioFormat, TranscriptionConfig, TranslationConfig};
use crate::realtime::SessionConfig;

/// returns TranscriptionConfig
pub fn get_transcription_config() -> TranscriptionConfig {
    TranscriptionConfig { 
        language: "en".to_string(), 
        domain: None, 
        output_locale: None, 
        operating_point: None, 
        additional_vocab: None, 
        punctuation_overrides: None, 
        diarization: None,
        enable_entities: None, 
        max_delay_mode: None, 
        speaker_diarization_config: None,
        enable_partials: None,
        max_delay: None,
        speaker_change_sensitivity: None, 
    }
}

/// returns TranslationConfig
pub fn get_translation_config() -> TranslationConfig {
    TranslationConfig { 
        enable_partials: None, 
        target_languages: [].to_vec() 
    }
}

/// returns SessionConfig
pub fn get_session_config() -> SessionConfig {
    SessionConfig::new(
        Some(get_transcription_config()), 
        Some(get_translation_config()), 
        Some(get_audio_format())
    )   
}

/// returns AudioFormat
pub fn get_audio_format() -> AudioFormat {
    AudioFormat{ 
        encoding: Some(models::audio_format::Encoding::PcmS16le), 
        sample_rate: Some(16000), 
        type_value: models::audio_format::Type::Raw 
    }
}