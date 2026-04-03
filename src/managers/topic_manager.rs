use std::sync::Arc;

use dashmap::DashMap;

use crate::queue::in_memory::ShardedQueue;

pub struct TopicManager {
    pub topics: DashMap<String, Arc<ShardedQueue>>,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            topics: DashMap::new(),
        }
    }
    pub fn get_or_create(&self, topic: String) -> Arc<ShardedQueue> {
        self.topics
            .entry(topic)
            .or_insert_with(|| Arc::new(ShardedQueue::new(8, 100_000)))
            .clone()
    }
}
