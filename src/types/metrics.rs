use std::sync::atomic::AtomicU64;

use serde::Serialize;

#[derive(Serialize)]
pub struct MetricsResponse {
    pub received: u64,
    pub delivered: u64,
}

pub struct Metrics {
    pub received: AtomicU64,
    pub delivered: AtomicU64,
}
