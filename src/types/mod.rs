use serde::{
    Deserialize,
    Serialize,
};

#[derive(Deserialize, Debug)]
pub struct ExecutionPayload {
    pub language: String,
    pub code: String,
    pub input: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ExecutionResult {
    pub output: String,
    pub error: String,
}
