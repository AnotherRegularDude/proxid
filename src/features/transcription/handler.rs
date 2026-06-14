use axum::Json;
use axum::extract::{Multipart, State};

use crate::app::AppState;
use crate::core::error::AppResult;
use crate::infrastructure::transcoder::Transcoder;

use super::dto::TranscriptionResponseBody;
use super::extractor::parse_transcription_form;
use super::service::TranscriptionService;

#[tracing::instrument(skip_all)]
pub async fn transcriptions<T: Transcoder>(
    State(state): State<AppState<T>>,
    multipart: Multipart,
) -> AppResult<Json<TranscriptionResponseBody>> {
    let form = parse_transcription_form(multipart, &state).await?;

    let svc = TranscriptionService::from_state(&state);
    let out = svc.run(form).await?;

    Ok(Json::from(TranscriptionResponseBody::new(out.text)))
}
