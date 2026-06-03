mod common;

use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;

use common::{response_body, spawn_app};

#[tokio::test]
async fn health_returns_200() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder().uri("/health").body(Body::empty()).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn root_returns_text() {
    let server = mockito::Server::new_async().await;
    let app = spawn_app(&server.url()).await;

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let (status, body) = response_body(resp).await;
    assert_eq!(status, 200);
    assert!(body.contains("proxid"));
}
