use std::sync::{Arc, atomic::Ordering};

use axum::{Json, extract::State};

use crate::{AppState, types::metrics::MetricsResponse};

pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        received: state.metrics.received.load(Ordering::Relaxed),
        delivered: state.metrics.delivered.load(Ordering::Relaxed),
    })
}
