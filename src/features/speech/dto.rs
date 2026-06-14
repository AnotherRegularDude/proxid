use std::str::FromStr;

use serde::Deserialize;

use crate::app::AppState;
use crate::core::audio::AudioFormat;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::transcoder::Transcoder;

#[derive(Deserialize)]
pub struct SpeechRequestBody {
    pub model: Option<String>,
    pub input: String,
    pub voice: String,
    pub response_format: Option<String>,
    pub speed: Option<f32>,
}

#[derive(Debug)]
pub struct SpeechRequest {
    pub model: String,
    pub input: String,
    pub voice: String,
    pub response_format: AudioFormat,
    pub speed: Option<f32>,
}

impl SpeechRequest {
    pub fn new(
        model: String,
        input: String,
        voice: String,
        response_format: AudioFormat,
        speed: Option<f32>,
    ) -> Self {
        Self { model, input, voice, response_format, speed }
    }
}

impl SpeechRequestBody {
    #[cfg(test)]
    pub fn new(
        model: Option<String>,
        input: String,
        voice: String,
        response_format: Option<String>,
        speed: Option<f32>,
    ) -> Self {
        Self { model, input, voice, response_format, speed }
    }

    pub fn into_request<T: Transcoder>(self, state: &AppState<T>) -> AppResult<SpeechRequest> {
        let model = self
            .model
            .unwrap_or_else(|| state.settings().provider().default_speech_model().clone());

        let response_format = match self.response_format.as_deref() {
            None => AudioFormat::Mp3,
            Some(fmt) => AudioFormat::from_str(fmt)
                .map_err(|_| AppError::BadRequest(format!("invalid response_format: {fmt}")))?,
        };

        if let Some(speed) = self.speed
            && !(0.25..=4.0).contains(&speed)
        {
            return Err(AppError::BadRequest(format!(
                "speed must be between 0.25 and 4.0, got {speed}"
            )));
        }

        if self.input.is_empty() {
            return Err(AppError::MissingField("input"));
        }

        if self.voice.is_empty() {
            return Err(AppError::MissingField("voice"));
        }

        Ok(SpeechRequest::new(model, self.input, self.voice, response_format, self.speed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState<crate::infrastructure::transcoder::FfmpegTranscoder> {
        let toml = r#"
[server]
host = "127.0.0.1"
port = 0

[provider]
base_url = "https://example.com"
api_key = "sk-test"
default_transcription_model = "test/stt"
default_speech_model = "test/tts"
request_timeout_secs = 5

[audio]
stt_sample_rate = 16000
aac_bitrate_bps = 160000
mp3_bitrate_bps = 128000
opus_bitrate_bps = 64000
pcm_sample_rate = 24000

[logging]
filter = "proxid=warn"
"#;
        let settings = crate::config::ConfigBuilder::new()
            .with_custom_config(Some(crate::config::MergedData::Content(toml.to_string())))
            .load()
            .expect("valid settings");

        crate::app::build_state(settings).expect("build state")
    }

    #[test]
    fn into_request_valid_with_defaults() {
        let state = test_state();
        let body =
            SpeechRequestBody::new(None, "hello".to_string(), "alloy".to_string(), None, None);
        let req = body.into_request(&state).unwrap();
        assert_eq!(req.model, "test/tts");
        assert_eq!(req.response_format, AudioFormat::Mp3);
        assert_eq!(req.input, "hello");
        assert_eq!(req.voice, "alloy");
        assert!(req.speed.is_none());
    }

    #[test]
    fn into_request_invalid_format_returns_bad_request() {
        let state = test_state();
        let body = SpeechRequestBody::new(
            None,
            "hello".to_string(),
            "alloy".to_string(),
            Some("xyz".to_string()),
            None,
        );
        let err = body.into_request(&state).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn into_request_speed_too_low_returns_bad_request() {
        let state = test_state();
        let body =
            SpeechRequestBody::new(None, "hello".to_string(), "alloy".to_string(), None, Some(0.1));
        let err = body.into_request(&state).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn into_request_speed_too_high_returns_bad_request() {
        let state = test_state();
        let body =
            SpeechRequestBody::new(None, "hello".to_string(), "alloy".to_string(), None, Some(5.0));
        let err = body.into_request(&state).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn into_request_speed_at_lower_bound_is_valid() {
        let state = test_state();
        let body = SpeechRequestBody::new(
            None,
            "hello".to_string(),
            "alloy".to_string(),
            None,
            Some(0.25),
        );
        let req = body.into_request(&state).unwrap();
        assert_eq!(req.speed, Some(0.25));
    }

    #[test]
    fn into_request_speed_at_upper_bound_is_valid() {
        let state = test_state();
        let body =
            SpeechRequestBody::new(None, "hello".to_string(), "alloy".to_string(), None, Some(4.0));
        let req = body.into_request(&state).unwrap();
        assert_eq!(req.speed, Some(4.0));
    }

    #[test]
    fn into_request_empty_voice_returns_missing_field() {
        let state = test_state();
        let body = SpeechRequestBody::new(None, "hello".to_string(), String::new(), None, None);
        let err = body.into_request(&state).unwrap_err();
        assert!(matches!(err, AppError::MissingField("voice")));
    }

    #[test]
    fn into_request_empty_input_returns_missing_field() {
        let state = test_state();
        let body = SpeechRequestBody::new(None, String::new(), "alloy".to_string(), None, None);
        let err = body.into_request(&state).unwrap_err();
        assert!(matches!(err, AppError::MissingField("input")));
    }
}
