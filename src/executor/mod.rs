mod k8s_executor;
pub use k8s_executor::K8sExecutor;
mod simple_executor;
pub use simple_executor::SimpleExecutor;

#[async_trait::async_trait]
pub trait CodeExecutor {
    async fn execute(
        payload: &super::types::ExecutionPayload,
    ) -> Result<super::types::ExecutionResult, Box<dyn std::error::Error>>;
}
