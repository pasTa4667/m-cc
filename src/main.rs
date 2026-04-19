use std::sync::{Arc, atomic::AtomicU64};

use m_cc::{AppState, app, managers::topic_manager::TopicManager, types::metrics::Metrics};

#[tokio::main]
async fn main() {
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());
    let state = Arc::new(AppState {
        topic_manager,
        metrics,
    });

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
