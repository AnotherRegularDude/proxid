use crate::app::AppState;
use crate::core::audio::{AudioFormat, SourceAudio};
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::openrouter::{TranscribePayload, TranscriptPayload};
use crate::infrastructure::transcoder::Transcoder;

use super::extractor::TranscriptionForm;

pub struct TranscriptionService<T: Transcoder> {
    provider: std::sync::Arc<crate::infrastructure::openrouter::OpenRouterClient>,
    transcoder: std::sync::Arc<T>,
}

impl<T: Transcoder> TranscriptionService<T> {
    pub fn from_state(state: &AppState<T>) -> Self {
        Self { provider: state.provider().clone(), transcoder: state.transcoder().clone() }
    }

    #[tracing::instrument(skip_all, fields(model = %form.model))]
    pub async fn run(&self, form: TranscriptionForm) -> AppResult<TranscriptPayload> {
        // Step 1: Transcode audio to WAV format (provider-ready)
        let src = SourceAudio::new(form.audio.bytes, form.audio.format);
        let wav_src: SourceAudio = self
            .transcoder
            .convert(src, AudioFormat::Wav)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        // Step 2: Send to provider (raw WAV bytes — client handles Base64 encoding)
        let input = TranscribePayload::new(
            wav_src.bytes.clone(),
            crate::core::audio::AudioFormat::Wav,
            Some(form.model),
            form.temperature,
        );

        let out = self
            .provider
            .transcribe(input)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        Ok(out)
    }
}
