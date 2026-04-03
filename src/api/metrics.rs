use std::sync::{Arc, atomic::Ordering};

use axum::{Json, extract::State};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct MetricsResponse {
    received: u64,
    delivered: u64,
}

pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        received: state.metrics.received.load(Ordering::Relaxed),
        delivered: state.metrics.delivered.load(Ordering::Relaxed),
    })
}
