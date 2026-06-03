use crate::app::AppState;
use crate::config::AudioConfig;
use crate::core::audio::{AudioFormat, OutputFormat};
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::openrouter::{OpenRouterClient, SpeechSynthRequest};
use crate::infrastructure::transcoder::{SourceAudio, Transcoder};

use super::dto::SpeechRequest;

pub(super) struct SpeechService<T: Transcoder> {
    provider: std::sync::Arc<OpenRouterClient>,
    transcoder: std::sync::Arc<T>,
    audio_cfg: AudioConfig,
}

impl<T: Transcoder> SpeechService<T> {
    pub fn from_state(state: &AppState<T>) -> Self {
        Self {
            provider: state.provider().clone(),
            transcoder: state.transcoder().clone(),
            audio_cfg: state.settings().audio.clone(),
        }
    }

    #[tracing::instrument(skip_all, fields(model = %req.model, voice = %req.voice, format = %req.response_format.as_ref()))]
    pub async fn run(&self, req: SpeechRequest) -> AppResult<(AudioFormat, bytes::Bytes)> {
        let synth = self
            .provider
            .synthesise(SpeechSynthRequest {
                model: req.model,
                input: req.input,
                voice: req.voice,
                speed: req.speed,
            })
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        // OpenRouter always returns PCM 24kHz/16-bit/mono; skip re-encoding when
        // the client requests PCM and the configured sample rate matches.
        if matches!(req.response_format, AudioFormat::Pcm)
            && self.audio_cfg.pcm_sample_rate == 24_000
        {
            return Ok((AudioFormat::Pcm, synth.bytes));
        }

        // SAFETY: into_request() already validated that response_format is
        // supported for speech, so for_speech() is guaranteed to return Some.
        let target = OutputFormat::for_speech(req.response_format)
            .expect("validated response_format should map to OutputFormat");

        let src = SourceAudio { bytes: synth.bytes, format: AudioFormat::Pcm };
        let bytes = self
            .transcoder
            .convert(src, target)
            .await
            .map_err(|e| AppError::Internal(format!("{e:#}")))?;

        Ok((req.response_format, bytes))
    }
}
