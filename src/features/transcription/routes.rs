use axum::Router;
use axum::routing::post;

use crate::app::AppState;
use crate::infrastructure::transcoder::Transcoder;

use super::handler;

pub fn routes<T: Transcoder>() -> Router<AppState<T>> {
    Router::new().route("/api/v1/audio/transcriptions", post(handler::transcriptions::<T>))
}
