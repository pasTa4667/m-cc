use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushParams {
    pub topic: Option<String>,
}

#[derive(Deserialize)]
pub struct PullParams {
    pub batch: Option<usize>,
    pub topic: Option<String>,
}
