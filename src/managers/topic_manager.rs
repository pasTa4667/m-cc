use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;

use crate::{config::WriterConfig, log::log::Log};

pub struct TopicManager {
    pub topic_logs: DashMap<String, Arc<Log>>,
    pub base_path: PathBuf,
    pub writer_config: WriterConfig,
}

impl TopicManager {
    pub fn new(path: &str, writer_config: WriterConfig) -> Self {
        let base_path = PathBuf::from(path);
        Self {
            topic_logs: DashMap::new(),
            base_path,
            writer_config,
        }
    }

    pub fn get_or_create(&self, topic: &str) -> Arc<Log> {
        self.topic_logs
            .entry(topic.to_string())
            .or_insert_with(|| {
                Arc::new(Log::new(
                    self.base_path.join(topic).as_path(),
                    self.writer_config,
                ))
            })
            .clone()
    }

    pub fn get(&self, topic: &str) -> Option<Arc<Log>> {
        if self.topic_logs.contains_key(topic) {
            return Some(self.get_or_create(topic));
        }

        None
    }
}
