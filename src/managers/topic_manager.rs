use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;

use crate::log::log::Log;

pub struct TopicManager {
    pub topic_logs: DashMap<String, Arc<Log>>,
    pub base_path: PathBuf,
}

impl TopicManager {
    pub fn new() -> Self {
        let base_path = PathBuf::from("E:\\Felix\\Programming\\ReactProjects\\m-cc\\target\\tmp");
        Self {
            topic_logs: DashMap::new(),
            base_path,
        }
    }
    pub fn get_or_create(&self, topic: &str) -> Arc<Log> {
        self.topic_logs
            .entry(topic.to_string())
            .or_insert_with(|| Arc::new(Log::new(self.base_path.join(topic).as_path())))
            .clone()
    }

    pub fn get(&self, topic: &str) -> Option<Arc<Log>> {
        if self.topic_logs.contains_key(topic) {
            return Some(self.get_or_create(topic));
        }

        None
    }
}
