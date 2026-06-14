use super::{Usage, audio::AudioFormat};

use bytes::Bytes;

#[derive(Debug)]
pub struct TranscribePayload {
    pub audio_bytes: Bytes,
    pub format: AudioFormat,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

pub struct TranscriptPayload {
    pub text: String,
    pub usage: Option<Usage>,
}

pub struct SpeechPayload {
    pub model: Option<String>,
    pub input: String,
    pub voice: String,
    pub speed: Option<f32>,
}

impl TranscribePayload {
    pub fn new(
        audio_bytes: Bytes,
        format: AudioFormat,
        model: Option<String>,
        temperature: Option<f32>,
    ) -> Self {
        Self { audio_bytes, format, model, temperature }
    }
}

impl TranscriptPayload {
    pub fn new(text: String, usage: Option<Usage>) -> Self {
        Self { text, usage }
    }
}

impl SpeechPayload {
    pub fn new(model: Option<String>, input: String, voice: String, speed: Option<f32>) -> Self {
        Self { model, input, voice, speed }
    }
}
