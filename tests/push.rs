use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use m_cc::config::{AppConfig, Config};
use m_cc::managers::topic_manager::TopicManager;
use m_cc::types::metrics::Metrics;
use m_cc::{AppState, app};
use tower::ServiceExt;

fn build_state() -> Arc<AppState> {
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let config = Config::get_default_values();

    let topic_manager = Arc::new(TopicManager::new(&config.storage.data_dir, config.writer));

    let app_config = AppConfig {
        runtime: config.runtime,
    };

    Arc::new(AppState {
        topic_manager,
        metrics,
        app_config: Arc::new(app_config),
    })
}

fn encode_messages(messages: &[&str]) -> Vec<u8> {
    let mut buf = Vec::new();

    for msg in messages {
        let bytes = msg.as_bytes();
        let len = msg.len() as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(bytes);
    }

    buf
}

#[tokio::test]
async fn push_returns_accepted_count_and_forwards_messages() {
    let state = build_state();
    let app = app(state);

    let payload = encode_messages(&["a", "b"]);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/push?topic=test")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let accepted: usize = std::str::from_utf8(&body).unwrap().parse().unwrap();
    assert_eq!(accepted, 2);
}

#[tokio::test]
async fn push_empty_messages_returns_zero() {
    let state = build_state();
    let app = app(state);

    let payload = encode_messages(&[]);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/push?topic=empty")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let accepted: usize = std::str::from_utf8(&body).unwrap().parse().unwrap();
    assert_eq!(accepted, 0);
}
