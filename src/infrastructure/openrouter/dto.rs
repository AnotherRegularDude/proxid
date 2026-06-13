use serde::{Deserialize, Serialize};

use crate::core::usage::Usage;

#[derive(Serialize, accessory::Accessors)]
#[access(get)]
pub struct TranscriptionRequestDto {
    model: String,
    input_audio: InputAudioDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, accessory::Accessors)]
#[access(get)]
pub struct InputAudioDto {
    data: String,
    format: &'static str,
}

#[derive(Deserialize, accessory::Accessors)]
#[access(get)]
pub struct TranscriptionResponseDto {
    text: String,
    usage: Option<UsageDto>,
}

#[derive(Deserialize, Clone, accessory::Accessors)]
#[access(get)]
pub struct UsageDto {
    seconds: Option<f64>,
    cost: Option<f64>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[derive(Serialize, accessory::Accessors)]
#[access(get, defaults(all(cp)))]
pub struct SpeechRequestDto<'a> {
    model: &'a str,
    input: &'a str,
    voice: &'a str,
    response_format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[derive(Deserialize, accessory::Accessors)]
#[access(get)]
pub struct ErrorResponseDto {
    error: ErrorDetailDto,
}

#[derive(Deserialize, accessory::Accessors)]
#[access(get)]
pub struct ErrorDetailDto {
    #[allow(
        dead_code,
        reason = "deserialized for API contract; may be used for error categorization"
    )]
    code: serde_json::Value,
    message: String,
}

impl TranscriptionRequestDto {
    pub fn new(model: String, input_audio: InputAudioDto, temperature: Option<f32>) -> Self {
        Self { model, input_audio, temperature }
    }
}

impl InputAudioDto {
    pub fn new(data: String, format: &'static str) -> Self {
        Self { data, format }
    }
}

impl<'a> SpeechRequestDto<'a> {
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

impl From<UsageDto> for Usage {
    fn from(u: UsageDto) -> Self {
        Self::new(*u.seconds(), *u.cost(), *u.input_tokens(), *u.output_tokens())
    }
}
