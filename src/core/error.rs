use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid request: {0}")]
    BadRequest(String),

    #[error("unsupported audio format: {0}")]
    UnsupportedFormat(String),

    #[error("missing field: {0}")]
    MissingField(&'static str),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorPayload,
}

#[derive(Serialize)]
struct ErrorPayload {
    message: String,
    r#type: &'static str,
}

impl AppError {
    pub fn create_internal_error() -> Self {
        AppError::Internal("Something went wrong".to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, kind) = match &self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "invalid_request_error"),
            AppError::UnsupportedFormat(_) => (StatusCode::BAD_REQUEST, "invalid_request_error"),
            AppError::MissingField(_) => (StatusCode::BAD_REQUEST, "invalid_request_error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        tracing::error!(error = %self, status = %status.as_u16(), kind, "request failed");

        let body = ErrorBody { error: ErrorPayload { message: self.to_string(), r#type: kind } };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    fn extract_status_and_type(error: AppError) -> (StatusCode, String) {
        let resp = error.into_response();
        let status = resp.status();
        let body = resp.into_body();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let bytes = rt.block_on(async { BodyExt::collect(body).await.unwrap().to_bytes() });
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let error_type = json["error"]["type"].as_str().unwrap().to_string();
        (status, error_type)
    }

    #[test]
    fn bad_request_maps_to_400() {
        let (status, kind) = extract_status_and_type(AppError::BadRequest(String::from("test")));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(kind, "invalid_request_error");
    }

    #[test]
    fn unsupported_format_maps_to_400() {
        let (status, kind) =
            extract_status_and_type(AppError::UnsupportedFormat(String::from("xyz")));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(kind, "invalid_request_error");
    }

    #[test]
    fn missing_field_maps_to_400() {
        let (status, kind) = extract_status_and_type(AppError::MissingField("file"));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(kind, "invalid_request_error");
    }

    #[test]
    fn internal_maps_to_500() {
        let (status, kind) = extract_status_and_type(AppError::Internal(String::from("oops")));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(kind, "internal_error");
    }
}
