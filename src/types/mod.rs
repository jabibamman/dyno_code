use serde::{
    Deserialize,
    Serialize,
};

#[derive(Deserialize, Debug)]
pub struct ExecutionPayload {
    pub language: String,
    pub code: String,
    pub output_extension: String,
    pub input_file_path: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ExecutionResult {
    pub error: String,
    pub output: String,
    pub output_file_path: Option<String>,
    pub output_file_content: Option<String>,
}
