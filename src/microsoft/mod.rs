use cognitive_services_speech_sdk_rs::audio::{
    AudioConfig, AudioStreamFormat, PullAudioInputStream, PushAudioInputStream,
};
use cognitive_services_speech_sdk_rs::common::ProfanityOption;
// use cognitive_services_speech_sdk_rs::ffi::PropertyId_SpeechServiceResponse_PostProcessingOption;
use cognitive_services_speech_sdk_rs::speech::{SpeechConfig, SpeechRecognizer};
// use cognitive_services_speech_sdk_rs::ffi::phrase_list_grammar_add_phrase;
// use cognitive_services_speech_sdk_rs as msspeech;
use log::*;

/// Recognizer
pub fn set_callbacks(speech_recognizer: &mut SpeechRecognizer) {
    speech_recognizer
        .set_session_started_cb(|event| debug!(">set_session_started_cb {:?}", event))
        .unwrap();

    speech_recognizer
        .set_session_stopped_cb(|event| debug!(">set_session_stopped_cb {:?}", event))
        .unwrap();

    speech_recognizer
        .set_speech_start_detected_cb(|event| debug!(">set_speech_start_detected_cb {:?}", event))
        .unwrap();

    speech_recognizer
        .set_speech_end_detected_cb(|event| debug!(">set_speech_end_detected_cb {:?}", event))
        .unwrap();

    speech_recognizer
        .set_recognizing_cb(|event| info!(">set_recognizing_cb {:?}", event.result.text))
        .unwrap();

    speech_recognizer
        .set_recognized_cb(|event| debug!(">set_recognized_cb {:?}", event))
        .unwrap();

    speech_recognizer
        .set_canceled_cb(|event| debug!(">set_canceled_cb {:?}", event))
        .unwrap();
}

///creates speech recognizer from provided audio config and implicit speech config
/// created from MS subscription key hardcoded in sample file
pub fn speech_recognizer_from_audio_cfg(audio_config: AudioConfig, ms_config: MsConfig) -> SpeechRecognizer {
    let mut speech_config = SpeechConfig::from_subscription(
        ms_config.ms_subscription_key,
        ms_config.ms_service_region,
    )
    .unwrap();
    speech_config.set_property(cognitive_services_speech_sdk_rs::common::PropertyId::SpeechServiceResponsePostProcessingOption,"TrueText".to_string()).unwrap();
    // let phrase_list = PhraseListGrammar.FromRecognizer(recognizer);

    speech_config.enable_dictation().unwrap();
    speech_config.set_profanity_option(ProfanityOption::Removed).unwrap();

    let speech_recognizer = SpeechRecognizer::from_config(speech_config, audio_config).unwrap();
    speech_recognizer
}

/// MsConfig
pub struct MsConfig {
    /// ms region
    pub ms_service_region: String,
    /// ms key
    pub ms_subscription_key: String
}

/// creates speech recognizer from push input stream and MS speech subscription key
/// returns recognizer and also push stream so that data push can be initiated
pub fn speech_recognizer_from_push_stream(ms_config: MsConfig) -> (SpeechRecognizer, PushAudioInputStream) {
    let wave_format = AudioStreamFormat::get_wave_format_pcm(16000, None, None).unwrap();
    let push_stream = PushAudioInputStream::create_push_stream_from_format(wave_format).unwrap();
    let audio_config = AudioConfig::from_stream_input(&push_stream).unwrap();
    (speech_recognizer_from_audio_cfg(audio_config, ms_config), push_stream)
}

/// creates speech recognizer from pull input stream and MS speech subscription key
/// returns recognizer and also pull stream so that data push can be initiated
pub fn speech_recognizer_from_pull_stream(ms_config: MsConfig) -> (SpeechRecognizer, PullAudioInputStream) {
    let wave_format = AudioStreamFormat::get_wave_format_pcm(16000, None, None).unwrap();
    let pull_stream = PullAudioInputStream::from_format(&wave_format).unwrap();
    let audio_config = AudioConfig::from_stream_input(&pull_stream).unwrap();
    (speech_recognizer_from_audio_cfg(audio_config, ms_config), pull_stream)
}

/// creates speech recognizer from wav input file and MS speech subscription key
pub fn speech_recognizer_from_wav_file(wav_file: &str, ms_config: MsConfig) -> SpeechRecognizer {
    let audio_config = AudioConfig::from_wav_file_input(wav_file).unwrap();
    speech_recognizer_from_audio_cfg(audio_config, ms_config)
}

/// creates speech recognizer from default mic settings and MS speech subscription key
pub fn speech_recognizer_default_mic(ms_config: MsConfig) -> SpeechRecognizer {
    let audio_config = AudioConfig::from_default_microphone_input().unwrap();
    speech_recognizer_from_audio_cfg(audio_config, ms_config)
}
