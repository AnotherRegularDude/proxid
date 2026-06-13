use axum::body::Body;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use http_body_util::BodyExt;
use proxid::app;
use proxid::config::Settings;

pub fn test_settings(mock_url: &str) -> Settings {
    let toml_str = format!(
        r#"
[server]
host = "127.0.0.1"
port = 0

[provider]
base_url = "{mock_url}"
api_key = "sk-test-key"
default_transcription_model = "openai/whisper-large-v3"
default_speech_model = "test/tts-model"
request_timeout_secs = 5

[audio]
stt_sample_rate = 16000
aac_bitrate_bps = 160000
mp3_bitrate_bps = 128000
opus_bitrate_bps = 64000
pcm_sample_rate = 24000

[logging]
filter = "proxid=warn"
"#
    );
    let figment = Figment::new().merge(Toml::string(&toml_str));
    figment.extract::<Settings>().expect("valid test settings")
}

pub async fn spawn_app(mock_url: &str) -> axum::Router {
    let settings = test_settings(mock_url);
    let state = app::build_state(settings).expect("failed to build state");
    app::build_router(state)
}

pub async fn response_body(resp: axum::http::Response<Body>) -> (axum::http::StatusCode, String) {
    let status = resp.status();
    let body = resp.into_body();
    let bytes = BodyExt::collect(body).await.expect("failed to read body").to_bytes();
    let text = String::from_utf8(bytes.to_vec()).expect("body is not utf8");
    (status, text)
}

#[allow(dead_code, reason = "used only in speech integration tests")]
pub async fn response_bytes(
    resp: axum::http::Response<Body>,
) -> (axum::http::StatusCode, bytes::Bytes, axum::http::HeaderValue) {
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| axum::http::HeaderValue::from_static("application/octet-stream"));
    let body = resp.into_body();
    let bytes = BodyExt::collect(body).await.expect("failed to read body").to_bytes();
    (status, bytes, content_type)
}

/// Reads a binary fixture from `tests/fixtures/<name>` and returns it as `Bytes`.
#[allow(dead_code, reason = "test utility used across multiple integration test files")]
pub fn load_fixture(name: &str) -> bytes::Bytes {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture '{}': {e}", path.display()))
        .into()
}

/// Build a multipart/form-data body for transcription tests.
///
/// `file_part` is `Option<(field_name, filename, content_type, data)>`.
/// `text_parts` is `&[(field_name, value)]` for optional text fields like "model" and "temperature".
#[allow(dead_code, reason = "test utility used across transcription integration tests")]
pub fn make_multipart_body(
    boundary: &str,
    file_part: Option<(&str, &str, &str, &[u8])>,
    text_parts: &[(&str, &str)],
) -> Vec<u8> {
    let mut body = Vec::new();

    if let Some((field_name, filename, content_type, data)) = file_part {
        let header = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\n\
             Content-Type: {content_type}\r\n\r\n"
        );
        body.extend_from_slice(header.as_bytes());
        body.extend_from_slice(data);
    }

    for (name, value) in text_parts {
        let part = format!(
            "\r\n--{boundary}\r\n\
             Content-Disposition: form-data; name=\"{name}\"\r\n\r\n\
             {value}"
        );
        body.extend_from_slice(part.as_bytes());
    }

    let closing = format!("\r\n--{boundary}--\r\n");
    body.extend_from_slice(closing.as_bytes());

    body
}
