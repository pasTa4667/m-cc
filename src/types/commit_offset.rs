use serde::Deserialize;

use crate::managers::offset_manager::OffsetKey;

#[derive(Deserialize)]
pub struct CommitOffsetParams {
    pub topic: String,
    pub group_id: Option<String>,
    pub offset: usize,
}

impl Into<OffsetKey> for CommitOffsetParams {
    fn into(self) -> OffsetKey {
        OffsetKey {
            topic: self.topic,
            group_id: self.group_id,
        }
    }
}
