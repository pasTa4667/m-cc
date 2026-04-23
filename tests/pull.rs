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

fn decode_messages(body: &[u8]) -> Vec<String> {
    let mut messages = Vec::new();
    let mut offset = 0;

    while offset + 4 <= body.len() {
        let len = u32::from_be_bytes(body[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + len > body.len() {
            break;
        }

        let msg = body[offset..offset + len].to_vec();
        messages.push(String::from_utf8(msg).unwrap());
        offset += len;
    }

    messages
}

#[tokio::test]
async fn pull_empty_queue_returns_empty_messages() {
    let state = build_state();
    let app = app(state);
    let topic = "empty-topic";

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/push?topic={}", topic))
                .header("content-type", "application/octet-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/pull?batch=10&topic={}", topic))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed = decode_messages(&body);
    assert!(parsed.is_empty());
}

#[tokio::test]
async fn pull_returns_messages_after_push() {
    let state = build_state();
    let app = app(state);
    let topic = "empty-topic";

    let push_payload = encode_messages(&["first", "second"]);
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/push?topic={}", topic))
                .header("content-type", "application/json")
                .body(Body::from(push_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/pull?batch=10&topic={}", topic))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed = decode_messages(&body);
    assert_eq!(parsed, vec!["first".to_string(), "second".to_string()]);
}
