pub mod tasks;
pub mod users;

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

use crate::db::RepositoryError;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error = self.to_string();
        let response = ErrorResponse { error };
        
        match self {
            ApiError::NotFound(_) => HttpResponse::NotFound().json(response),
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(response),
            ApiError::Internal(_) => HttpResponse::InternalServerError().json(response),
        }
    }
}

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => ApiError::NotFound(msg),
            RepositoryError::Database(err) => ApiError::Internal(format!("Database error: {}", err)),
            RepositoryError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}
