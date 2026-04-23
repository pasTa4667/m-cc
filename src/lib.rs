use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::{metrics::metrics_handler, pull::pull_handler, push::push_handler},
    config::AppConfig,
    managers::topic_manager::TopicManager,
    types::metrics::Metrics,
};

pub mod api;
pub mod enums;
pub mod log;
pub mod managers;
pub mod queue;
pub mod types;

pub mod config;

pub struct AppState {
    pub topic_manager: Arc<TopicManager>,
    pub metrics: Arc<Metrics>,
    pub app_config: Arc<AppConfig>,
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/push", post(push_handler))
        .route("/pull", get(pull_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}
