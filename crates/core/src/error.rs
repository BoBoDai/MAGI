//! Domain errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Llm(String),
    #[error("模型输出不合法: {0}")]
    InvalidLlmOutput(String),
    #[error("存储失败: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, Error>;
