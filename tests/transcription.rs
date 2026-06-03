mod common;

use axum::body::Body;
use axum::http::{Request, header};
use tower::ServiceExt;

use common::{load_fixture, response_body, spawn_app};

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
    let body_prefix = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.wav\"\r\n\
         Content-Type: audio/wav\r\n\r\n"
    );
    let body_suffix = format!(
        "\r\n--{boundary}\r\n\
         Content-Disposition: form-data; name=\"model\"\r\n\r\n\
         openai/whisper-large-v3\r\n\
         --{boundary}--\r\n"
    );

    let full_body: Vec<u8> =
        body_prefix.bytes().chain(wav_data.iter().copied()).chain(body_suffix.bytes()).collect();

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
    let body_str = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"model\"\r\n\r\n\
         openai/whisper-large-v3\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body_str))
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
    let body_str = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.xyz\"\r\n\
         Content-Type: application/octet-stream\r\n\r\n\
         some data\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .uri("/api/v1/audio/transcriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body_str))
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
    let body_str = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.wav\"\r\n\
         Content-Type: audio/wav\r\n\r\n"
    );
    let full_body: Vec<u8> = body_str
        .bytes()
        .chain(wav_data.iter().copied())
        .chain(format!("\r\n--{boundary}--\r\n").bytes())
        .collect();

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
    let body_str = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.wav\"\r\n\
         Content-Type: audio/wav\r\n\r\n"
    );
    let full_body: Vec<u8> = body_str
        .bytes()
        .chain(wav_data.iter().copied())
        .chain(format!("\r\n--{boundary}--\r\n").bytes())
        .collect();

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
    let body_prefix = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"sample.wav\"\r\n\
         Content-Type: audio/wav\r\n\r\n"
    );
    let body_suffix = format!(
        "\r\n--{boundary}\r\n\
         Content-Disposition: form-data; name=\"model\"\r\n\r\n\
         openai/whisper-large-v3\r\n\
         --{boundary}--\r\n"
    );

    let full_body: Vec<u8> =
        body_prefix.bytes().chain(wav_data.iter().copied()).chain(body_suffix.bytes()).collect();

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
    let body_prefix = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"sample.mp3\"\r\n\
         Content-Type: audio/mpeg\r\n\r\n"
    );
    let body_suffix = format!(
        "\r\n--{boundary}\r\n\
         Content-Disposition: form-data; name=\"model\"\r\n\r\n\
         openai/whisper-large-v3\r\n\
         --{boundary}--\r\n"
    );

    let full_body: Vec<u8> =
        body_prefix.bytes().chain(mp3_data.iter().copied()).chain(body_suffix.bytes()).collect();

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
