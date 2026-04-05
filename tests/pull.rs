use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use m_cc::managers::topic_manager::TopicManager;
use m_cc::queue::in_memory::ShardedQueue;
use m_cc::types::metrics::Metrics;
use m_cc::{AppState, app};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn pull_empty_queue_returns_empty_messages() {
    let message_queue = Arc::new(ShardedQueue::new(8, 100_000));
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());
    let state = Arc::new(AppState {
        queue: message_queue,
        topic_manager,
        metrics,
    });
    let app = app(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/pull")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["messages"], json!([]));
}

#[tokio::test]
async fn pull_returns_messages_after_push() {
    let message_queue = Arc::new(ShardedQueue::new(8, 100_000));
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());
    let state = Arc::new(AppState {
        queue: message_queue,
        topic_manager,
        metrics,
    });
    let app = app(state);

    let push_payload = json!({ "messages": ["first", "second"] });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/push")
                .header("content-type", "application/json")
                .body(Body::from(push_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/pull?batch=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["messages"], json!(["first", "second"]));
}
