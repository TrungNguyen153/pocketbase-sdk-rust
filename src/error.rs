use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PocketBaseErrorResponse {
    code: u8,
    message: String,
    data: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Error {
    /// An invalid request parameter
    InvalidParameter(Box<dyn std::error::Error + Send + Sync + 'static>),
    RequestFailed(Box<dyn std::error::Error + Send + Sync + 'static>),
    NotAuthenticated,
    AuthenticationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    PocketBaseErrorResponse(PocketBaseErrorResponse),
    PocketBaseImplementException(String),
    Timeout(String),
    SSEClientNotExist,
    ShouldNot(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidParameter(err) => write!(f, "Invalid parameter: {err}"),
            RequestFailed(err) => write!(f, "Request failed: {err}"),
            AuthenticationError(err) => write!(f, "Authentication error: {err}"),
            PocketBaseErrorResponse(response_err) => {
                write!(f, "PocketBase request error: {response_err:#?}")
            }
            Timeout(reason) => write!(f, "Timeout: {reason}"),
            SSEClientNotExist => write!(f, "SSE Client not created yet"),
            NotAuthenticated => write!(f, "Not authenticated"),
            ShouldNot(reason) => write!(f, "Should not: {reason}"),
            PocketBaseImplementException(reason) => {
                write!(f, "Pocketbase implementation error: {reason}")
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
