mod common;

use axum::body::Body;
use axum::http::{Request, header};
use tower::ServiceExt;

use common::{load_fixture, make_multipart_body, response_body, spawn_app};

fn make_test_wav() -> Vec<u8> {
    common::load_fixture("sample.wav").to_vec()
}

#[tokio::test]
async fn transcription_happy_path() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/audio/transcriptions")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(r#"{"text":"hello world","usage":{"seconds":3.5,"cost":0.001}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "test.wav", "audio/wav", &wav_data)),
        &[("model", "openai/whisper-large-v3")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["text"], "hello world");

    mock.assert_async().await;
}

#[tokio::test]
async fn transcription_missing_file_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let full_body = make_multipart_body(boundary, None, &[("model", "openai/whisper-large-v3")]);

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn transcription_unsupported_format_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "test.xyz", "application/octet-stream", b"some data")),
        &[],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn transcription_upstream_401_maps_to_500() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/audio/transcriptions")
        .with_status(401)
        .with_body(r#"{"error":{"code":401,"message":"Unauthorized"}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body =
        make_multipart_body(boundary, Some(("file", "test.wav", "audio/wav", &wav_data)), &[]);

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 500);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "internal_error");
}

#[tokio::test]
async fn transcription_upstream_429_maps_to_500() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/audio/transcriptions")
        .with_status(429)
        .with_body(r#"{"error":{"code":429,"message":"Rate limited"}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body =
        make_multipart_body(boundary, Some(("file", "test.wav", "audio/wav", &wav_data)), &[]);
    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 500);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "internal_error");
}

#[tokio::test]
async fn transcription_happy_path_with_fixture_wav() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/audio/transcriptions")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(r#"{"text":"fixture hello","usage":{"seconds":1.0,"cost":0.002}}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let wav_data = load_fixture("sample.wav");
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "sample.wav", "audio/wav", &wav_data)),
        &[("model", "openai/whisper-large-v3")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["text"], "fixture hello");

    mock.assert_async().await;
}

#[tokio::test]
async fn transcription_custom_model_sent_to_provider() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/audio/transcriptions")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "custom/stt-model"
        })))
        .with_status(200)
        .with_body(r#"{"text":"custom model ok"}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "test.wav", "audio/wav", &wav_data)),
        &[("model", "custom/stt-model")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["text"], "custom model ok");

    mock.assert_async().await;
}

#[tokio::test]
async fn transcription_temperature_sent_to_provider() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/audio/transcriptions")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "temperature": 0.2
        })))
        .with_status(200)
        .with_body(r#"{"text":"temperature ok"}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "test.wav", "audio/wav", &wav_data)),
        &[("temperature", "0.2")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["text"], "temperature ok");

    mock.assert_async().await;
}

#[tokio::test]
async fn transcription_invalid_temperature_returns_400() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let wav_data = make_test_wav();
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "test.wav", "audio/wav", &wav_data)),
        &[("temperature", "not-a-number")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 400);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
}

#[tokio::test]
async fn transcription_mp3_input_works() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/audio/transcriptions")
        .match_header("Authorization", "Bearer sk-test-key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(r#"{"text":"mp3 transcription","usage":null}"#)
        .create_async()
        .await;

    let app = spawn_app(&server.url()).await;

    let mp3_data = load_fixture("sample.mp3");
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let full_body = make_multipart_body(
        boundary,
        Some(("file", "sample.mp3", "audio/mpeg", &mp3_data)),
        &[("model", "openai/whisper-large-v3")],
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(full_body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["text"], "mp3 transcription");

    mock.assert_async().await;
}
