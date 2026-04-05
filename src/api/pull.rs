use std::sync::{Arc, atomic::Ordering};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use bytes::Bytes;

use crate::{AppState, types::message::PullParams};

pub async fn pull_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullParams>,
) -> impl IntoResponse {
    let batch_size = params.batch.unwrap_or(100).min(1000);
    let messages: Vec<Bytes>;

    let default = "default";
    let consumer_id = params.consumer_id.as_deref().unwrap_or(default);

    let topic = params.topic.as_deref().unwrap_or(default);
    match state.topic_manager.get(topic) {
        Some(topic_log) => {
            messages = topic_log.read_batch(consumer_id, batch_size);
        }
        None => {
            return (StatusCode::NOT_FOUND).into_response();
        }
    }

    state
        .metrics
        .delivered
        .fetch_add(messages.len() as u64, Ordering::Relaxed);

    encode_messages(messages).into_response()
}

fn encode_messages(messages: Vec<Bytes>) -> Bytes {
    let mut buf = Vec::new();

    for msg in messages {
        let len = msg.len() as u32;

        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&msg);
    }

    Bytes::from(buf)
}
