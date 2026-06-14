use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;

use crate::config::Settings;
use crate::features::meta::routes::routes as meta_routes;
use crate::features::speech::routes::routes as speech_routes;
use crate::features::transcription::routes::routes as transcription_routes;
use crate::infrastructure::openrouter::OpenRouterClient;
use crate::infrastructure::transcoder::{FfmpegTranscoder, Transcoder};

/// Maximum request body size (25 MB) — accommodates large audio uploads.
const MAX_BODY_SIZE: usize = 25 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState<T: Transcoder + Clone = FfmpegTranscoder> {
    settings: Arc<Settings>,
    provider: Arc<OpenRouterClient>,
    transcoder: Arc<T>,
}

impl<T: Transcoder> AppState<T> {
    pub fn settings(&self) -> &Arc<Settings> {
        &self.settings
    }

    pub fn provider(&self) -> &Arc<OpenRouterClient> {
        &self.provider
    }

    pub fn transcoder(&self) -> &Arc<T> {
        &self.transcoder
    }
}

pub fn build_router<T: Transcoder>(state: AppState<T>) -> Router {
    Router::new()
        .merge(meta_routes::<T>())
        .merge(speech_routes::<T>())
        .merge(transcription_routes::<T>())
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

pub fn build_state(settings: Settings) -> anyhow::Result<AppState<FfmpegTranscoder>> {
    tracing::debug!(host = %settings.server().host(), port = %settings.server().port(), "building app state");

    let client = OpenRouterClient::builder()
        .base_url(settings.provider().base_url().clone())
        .api_key(settings.provider().api_key().into())
        .default_transcription_model(settings.provider().default_transcription_model().clone())
        .default_speech_model(settings.provider().default_speech_model().clone())
        .timeout_secs(settings.provider().request_timeout_secs())
        .app_referer(settings.provider().app_referer().clone())
        .app_name(settings.provider().app_name().clone())
        .build()?;

    let transcoder = FfmpegTranscoder::new(settings.audio().into())?;

    tracing::debug!(default_model = %settings.provider().default_transcription_model(), "provider client built");
    Ok(AppState {
        settings: Arc::new(settings),
        provider: Arc::new(client),
        transcoder: Arc::new(transcoder),
    })
}
