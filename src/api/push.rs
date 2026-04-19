use std::sync::{Arc, atomic::Ordering};

use crate::{AppState, types::message::PushParams};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use bytes::Bytes;

pub async fn push_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PushParams>,
    body: Bytes,
) -> impl IntoResponse {
    let msgs: Vec<Bytes> = parse_messages(body);

    let topic = params.topic.as_deref().unwrap_or("default");
    let accepted = state.topic_manager.get_or_create(topic).append_batch(msgs);

    state
        .metrics
        .received
        .fetch_add(accepted as u64, Ordering::Relaxed);

    accepted.to_string()
}

fn parse_messages(body: Bytes) -> Vec<Bytes> {
    let mut messages = Vec::new();
    let mut offset = 0;

    while offset + 4 <= body.len() {
        let len = u32::from_be_bytes(body[offset..offset + 4].try_into().unwrap()) as usize;

        offset += 4;

        if offset + len > body.len() {
            break;
        }

        let msg = body.slice(offset..offset + len);
        messages.push(msg);

        offset += len;
    }

    messages
}
