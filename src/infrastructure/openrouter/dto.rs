use serde::{Deserialize, Serialize};

use crate::core::usage::Usage;

#[derive(Serialize)]
pub struct TranscriptionRequestDto {
    pub model: String,
    pub input_audio: InputAudioDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Serialize)]
pub struct InputAudioDto {
    pub data: String,
    pub format: &'static str,
}

#[derive(Deserialize)]
pub struct TranscriptionResponseDto {
    pub text: String,
    pub usage: Option<UsageDto>,
}

#[derive(Deserialize)]
pub struct UsageDto {
    pub seconds: Option<f64>,
    pub cost: Option<f64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

impl From<UsageDto> for Usage {
    fn from(u: UsageDto) -> Self {
        Self {
            seconds: u.seconds,
            cost: u.cost,
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
        }
    }
}

#[derive(Serialize)]
pub struct SpeechRequestDto<'a> {
    pub model: &'a str,
    pub input: &'a str,
    pub voice: &'a str,
    pub response_format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

#[derive(Deserialize)]
pub struct ErrorResponseDto {
    pub error: ErrorDetailDto,
}

#[derive(Deserialize)]
pub struct ErrorDetailDto {
    #[allow(dead_code)]
    pub code: Option<i32>,
    pub message: String,
}
