use base64::prelude::*;
use bytes::Bytes;

use crate::app::AppState;
use crate::core::audio::OutputFormat;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::openrouter::TranscriptionInput;
use crate::infrastructure::openrouter::TranscriptionOutput;
use crate::infrastructure::transcoder::{SourceAudio, Transcoder};

use super::extractor::TranscriptionForm;

pub(super) struct TranscriptionService<T: Transcoder> {
    provider: std::sync::Arc<crate::infrastructure::openrouter::OpenRouterClient>,
    transcoder: std::sync::Arc<T>,
}

impl<T: Transcoder> TranscriptionService<T> {
    pub fn from_state(state: &AppState<T>) -> Self {
        Self { provider: state.provider().clone(), transcoder: state.transcoder().clone() }
    }

    #[tracing::instrument(skip_all, fields(model = %form.model))]
    pub async fn run(&self, form: TranscriptionForm) -> AppResult<TranscriptionOutput> {
        // Step 1: Transcode audio to STT WAV format (16 kHz, mono, s16le)
        let src = SourceAudio { bytes: form.audio.bytes, format: form.audio.format };
        let wav_bytes: Bytes = self
            .transcoder
            .convert(src, OutputFormat::SttWav)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Step 2: Base64-encode the WAV
        let audio_base64 = BASE64_STANDARD.encode(&wav_bytes);

        // Step 3: Send to provider
        let input = TranscriptionInput {
            audio_bytes: Bytes::from(audio_base64.into_bytes()),
            format: crate::core::audio::AudioFormat::Wav,
            model: form.model,
            temperature: form.temperature,
        };

        let out =
            self.provider.transcribe(input).await.map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(out)
    }
}
