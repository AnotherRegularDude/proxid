use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::app::AppState;
use crate::infrastructure::transcoder::Transcoder;

pub fn routes<T: Transcoder>() -> Router<AppState<T>> {
    Router::new().route("/", get(root)).route("/health", get(health))
}

async fn root() -> &'static str {
    "proxid — OpenAI-compatible audio proxy"
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}
