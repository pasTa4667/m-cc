use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::{metrics::metrics_handler, pull::pull_handler, push::push_handler},
    managers::topic_manager::TopicManager,
    queue::in_memory::ShardedQueue,
    types::metrics::Metrics,
};

pub mod api;
pub mod managers;
pub mod queue;
pub mod types;

pub struct AppState {
    pub queue: Arc<ShardedQueue>,
    pub topic_manager: Arc<TopicManager>,
    pub metrics: Arc<Metrics>,
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/push", post(push_handler))
        .route("/pull", get(pull_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}
