//! Types for PPS api.
//! See `api/docs` for documentation (TODO).
use serde::{Deserialize, Serialize};

/// Returned by endpoints which represent long-running operations.
#[derive(Serialize, Deserialize)]
pub struct OperationInfo {
    /// Unique identifier of the started operation.
    pub id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationStatus {
    Running,
    Failed,
    Completed,
}

/// Represents operation.
#[derive(Serialize, Deserialize)]
pub struct Operation {
    /// Identifier of the operation
    pub id: uuid::Uuid,
    /// List of updates
    pub events: Vec<serde_json::Value>,
    /// Operation status
    pub status: OperationStatus,
    /// Error (exists when status is FAILED)
    pub error: Option<String>,
}

/// Api error.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiError {
    pub kind: ErrorKind,
    pub code: String,
    pub details: serde_json::Value,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "api error: {} ({}): {:?}",
            self.code,
            self.kind.string(),
            self.details
        )
    }
}

/// Error kind
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    NotFound,
    Internal,
}

impl ErrorKind {
    pub fn http_status(self) -> u16 {
        match self {
            ErrorKind::NotFound => 404,
            ErrorKind::Internal => 500,
        }
    }

    pub fn string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
