use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::{StatusCode, header};
use axum::response::Response;

use crate::app::AppState;
use crate::core::error::AppResult;
use crate::infrastructure::transcoder::Transcoder;

use super::dto::SpeechRequestBody;
use super::service::SpeechService;

#[tracing::instrument(skip_all)]
pub async fn speech<T: Transcoder>(
    State(state): State<AppState<T>>,
    Json(body): Json<SpeechRequestBody>,
) -> AppResult<Response> {
    let req = body.into_request(&state)?;

    let svc = SpeechService::from_state(&state);
    let audio = svc.run(req).await?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, audio.format.mime())
        .body(Body::from(audio.bytes.clone()))
        .map_err(|e| crate::core::error::AppError::Internal(format!("{e:#}")))?;

    Ok(resp)
}
