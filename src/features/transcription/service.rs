use bytes::Bytes;

use crate::app::AppState;
use crate::core::audio::OutputFormat;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::openrouter::{TranscriptionInput, TranscriptionOutput};
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

    #[tracing::instrument(skip_all, fields(model = %form.model()))]
    pub async fn run(&self, form: TranscriptionForm) -> AppResult<TranscriptionOutput> {
        // Step 1: Transcode audio to STT WAV format (16 kHz, mono, s16le)
        let src = SourceAudio::new(form.audio().bytes().clone(), *form.audio().format());
        let wav_bytes: Bytes = self
            .transcoder
            .convert(src, OutputFormat::SttWav)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        // Step 2: Send to provider (raw WAV bytes — client handles Base64 encoding)
        let input = TranscriptionInput::new(
            wav_bytes,
            crate::core::audio::AudioFormat::Wav,
            form.model().clone(),
            *form.temperature(),
        );

        let out = self
            .provider
            .transcribe(input)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        Ok(out)
    }
}
