use std::sync::Arc;

use crate::app::AppState;
use crate::config::AudioConfig;
use crate::core::audio::{AudioFormat, SourceAudio};
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::openrouter::{OpenRouterClient, SpeechPayload};
use crate::infrastructure::transcoder::Transcoder;

use super::dto::SpeechRequest;

pub struct SpeechService<T: Transcoder> {
    provider: Arc<OpenRouterClient>,
    transcoder: Arc<T>,
    audio_cfg: AudioConfig,
}

impl<T: Transcoder> SpeechService<T> {
    pub fn from_state(state: &AppState<T>) -> Self {
        Self {
            provider: state.provider().clone(),
            transcoder: state.transcoder().clone(),
            audio_cfg: state.settings().audio().clone(),
        }
    }

    #[tracing::instrument(skip_all, fields(model = %req.model, voice = %req.voice, format = %req.response_format.as_ref()))]
    pub async fn run(&self, req: SpeechRequest) -> AppResult<SourceAudio> {
        let synth = self
            .provider
            .synthesise(SpeechPayload::new(Some(req.model), req.input, req.voice, req.speed))
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        // OpenRouter always returns PCM 24kHz/16-bit/mono; skip re-encoding when
        // the client requests PCM and the configured sample rate matches.
        if matches!(req.response_format, AudioFormat::Pcm)
            && self.audio_cfg.pcm_sample_rate() == 24_000
        {
            return Ok(synth);
        }

        let out = self
            .transcoder
            .convert(synth, req.response_format)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        Ok(out)
    }
}
