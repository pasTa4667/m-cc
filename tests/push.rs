use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use m_cc::managers::topic_manager::TopicManager;
use m_cc::types::metrics::Metrics;
use m_cc::{AppState, app};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn push_returns_accepted_count_and_forwards_messages() {
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());
    let state = Arc::new(AppState {
        topic_manager,
        metrics,
    });
    let app = app(state);

    let payload = json!({ "messages": ["a", "b"] });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/push")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let accepted: usize = serde_json::from_slice(&body).unwrap();
    assert_eq!(accepted, 2);
}

#[tokio::test]
async fn push_empty_messages_returns_zero() {
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());
    let state = Arc::new(AppState {
        topic_manager,
        metrics,
    });
    let app = app(state);

    let payload = json!({ "messages": [] });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/push")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let accepted: usize = serde_json::from_slice(&body).unwrap();
    assert_eq!(accepted, 0);
}
