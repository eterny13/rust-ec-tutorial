use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("Invalid status transition from {current} when trying to {action}")]
    InvalidStatusTransition { current: String, action: String },

    #[error("Validation error: {0}")]
    ValidationError(String),
}
