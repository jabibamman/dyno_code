use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ExecutionPayload {
    pub language: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct ExecutionResult {
    pub output: String,
    pub error: String,
}
