pub mod agent_error;

use std::io;

use thiserror::Error as ThisError;

use crate::error::agent_error::AgentError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("model error: {0}")]
    ModelError(#[from] model_gateway_rs::error::Error),

    #[error("agent error: {0}")]
    AgentError(#[from] AgentError),
}

pub type Result<T> = core::result::Result<T, Error>;
