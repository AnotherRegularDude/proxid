mod common;

use axum::body::Body;
use axum::http::{Request, header};
use tower::ServiceExt;

use common::{load_fixture, response_body, response_bytes, spawn_app};

#[tokio::test]
async fn speech_happy_path_pcm() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"pcm"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert_eq!(bytes.len(), pcm_data.len(), "PCM pass-through should preserve byte count");
    assert_eq!(content_type.to_str().unwrap(), "audio/L16");
}

#[tokio::test]
async fn speech_happy_path_mp3() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"mp3"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert!(!bytes.is_empty(), "mp3 body should not be empty");
    assert_eq!(content_type.to_str().unwrap(), "audio/mpeg");
}

#[tokio::test]
async fn speech_happy_path_opus() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"opus"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert!(!bytes.is_empty(), "opus body should not be empty");
    assert_eq!(content_type.to_str().unwrap(), "audio/opus");
}

#[tokio::test]
async fn speech_happy_path_aac() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"aac"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert!(!bytes.is_empty(), "aac body should not be empty");
    assert_eq!(content_type.to_str().unwrap(), "audio/aac");
}

#[tokio::test]
async fn speech_happy_path_flac() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"flac"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert!(!bytes.is_empty(), "flac body should not be empty");
    assert_eq!(content_type.to_str().unwrap(), "audio/flac");
}

#[tokio::test]
async fn speech_happy_path_wav() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = load_fixture("sample_pcm_24k.bin");
    let _mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"wav"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, bytes, content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);
    assert!(!bytes.is_empty(), "wav body should not be empty");
    assert_eq!(content_type.to_str().unwrap(), "audio/wav");
}

#[tokio::test]
async fn speech_uses_default_model_when_omitted() {
    let mut server = mockito::Server::new_async().await;

    let pcm_data = vec![0u8; 480];
    let mock = server
        .mock("POST", "/audio/speech")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(&pcm_data)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    // Request without "model" field — should use default_speech_model from settings
    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"pcm"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, _bytes, _content_type) = response_bytes(resp).await;
    assert_eq!(status, 200);

    mock.assert_async().await;
}

#[tokio::test]
async fn speech_bad_format_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"xyz"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn speech_bad_speed_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy","speed":5.0}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn speech_unsupported_format_for_speech_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        // m4a is a valid AudioFormat but not supported for speech
        .body(Body::from(r#"{"input":"hello","voice":"alloy","response_format":"m4a"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn speech_empty_input_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"","voice":"alloy"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn speech_empty_voice_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":""}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn speech_unauthorized_returns_500() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/audio/speech")
        .with_status(401)
        .with_body(r#"{"error":{"code":401,"message":"Unauthorized"}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 500);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "internal_error");
}

#[tokio::test]
async fn speech_rate_limited_returns_500() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/audio/speech")
        .with_status(429)
        .with_body(r#"{"error":{"code":429,"message":"Rate limited"}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 500);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "internal_error");
}

#[tokio::test]
async fn speech_upstream_500_returns_500() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/audio/speech")
        .with_status(500)
        .with_body(r#"{"error":{"code":500,"message":"Internal Server Error"}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let req = Request::builder()
        .uri("/api/v1/audio/speech")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"input":"hello","voice":"alloy"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 500);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "internal_error");
}
