use std::sync::atomic::AtomicU64;

pub struct Metrics {
    pub received: AtomicU64,
    pub delivered: AtomicU64,
}
