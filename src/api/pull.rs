use std::sync::{Arc, atomic::Ordering};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use bytes::Bytes;

use crate::{AppState, queue::ParallelQueue, types::message::PullParams};

pub async fn pull_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullParams>,
) -> impl IntoResponse {
    let batch_size = params.batch.unwrap_or(100).min(1000);
    let messages: Vec<Bytes>;

    if let Some(topic) = params.topic {
        let topic_queue = state.topic_manager.get_or_create(topic);

        messages = topic_queue.pop_batch_parallel(batch_size).await;
    } else {
        messages = state.queue.pop_batch_parallel(batch_size).await;
    }

    let accepted = messages.len();

    state
        .metrics
        .delivered
        .fetch_add(accepted as u64, Ordering::Relaxed);

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
