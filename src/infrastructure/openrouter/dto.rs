use serde::{Deserialize, Serialize};

use crate::core::{TranscriptPayload, Usage};

#[derive(Serialize)]
pub struct ProviderTranscribeRequest {
    pub model: String,
    pub input_audio: ProviderInputAudio,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Serialize)]
pub struct ProviderInputAudio {
    pub data: String,
    pub format: &'static str,
}

#[derive(Deserialize)]
pub struct ProviderTranscribeResponse {
    pub text: String,
    pub usage: Option<ProviderUsage>,
}

#[derive(Deserialize, Clone)]
pub struct ProviderUsage {
    pub seconds: Option<f64>,
    pub cost: Option<f64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Serialize)]
pub struct ProviderSpeechRequest<'a> {
    pub model: &'a str,
    pub input: &'a str,
    pub voice: &'a str,
    pub response_format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Deserialize)]
pub struct ErrorDetail {
    #[allow(
        dead_code,
        reason = "deserialized for API contract; may be used for error categorization"
    )]
    pub code: serde_json::Value,
    pub message: String,
}

impl ProviderTranscribeRequest {
    pub fn new(model: String, input_audio: ProviderInputAudio, temperature: Option<f32>) -> Self {
        Self { model, input_audio, temperature }
    }
}

impl ProviderInputAudio {
    pub fn new(data: String, format: &'static str) -> Self {
        Self { data, format }
    }
}

impl<'a> ProviderSpeechRequest<'a> {
    pub fn new(
        model: &'a str,
        input: &'a str,
        voice: &'a str,
        response_format: &'static str,
        speed: Option<f32>,
    ) -> Self {
        Self { model, input, voice, response_format, speed }
    }
}

impl From<ProviderTranscribeResponse> for TranscriptPayload {
    fn from(val: ProviderTranscribeResponse) -> Self {
        TranscriptPayload { text: val.text, usage: val.usage.map(|u| u.into()) }
    }
}

impl From<ProviderUsage> for Usage {
    fn from(val: ProviderUsage) -> Self {
        Usage::new(val.seconds, val.cost, val.input_tokens, val.output_tokens)
    }
}
