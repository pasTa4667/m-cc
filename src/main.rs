use std::sync::{Arc, atomic::AtomicU64};

use m_cc::{
    AppState, app, managers::topic_manager::TopicManager, queue::in_memory::ShardedQueue,
    types::metrics::Metrics,
};

#[tokio::main]
async fn main() {
    let message_queue = Arc::new(ShardedQueue::new(8, 100_000));
    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let topic_manager = Arc::new(TopicManager::new());

    let state = Arc::new(AppState {
        queue: message_queue,
        topic_manager,
        metrics,
    });

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
