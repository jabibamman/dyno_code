mod simple_executor;
pub use simple_executor::SimpleExecutor;

pub trait CodeExecutor {
    fn execute(payload: &super::types::ExecutionPayload) -> super::types::ExecutionResult;
}
