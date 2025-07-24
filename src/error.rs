use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lud06Error {
    status: String,
    reason: String,
}

impl Lud06Error {
    pub fn new(reason: String) -> Self {
        Lud06Error {
            status: "ERROR".to_string(),
            reason,
        }
    }
}

impl Display for Lud06Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.reason)
    }
}

impl From<anyhow::Error> for Lud06Error {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("{}", e.backtrace());
        Lud06Error {
            status: "ERROR".to_string(),
            reason: e.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HttpError {
    status_code: StatusCode,
    e: Lud06Error,
}

impl HttpError {
    pub fn new(status_code: StatusCode, e: Lud06Error) -> HttpError {
        HttpError { status_code, e }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let mut res = Json(self.e).into_response();
        *res.status_mut() = self.status_code;
        res
    }
}

impl From<anyhow::Error> for HttpError {
    fn from(e: anyhow::Error) -> Self {
        HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, e.into())
    }
}
