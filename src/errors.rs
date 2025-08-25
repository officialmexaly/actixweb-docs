use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(err) => {
                log::error!("Database error: {}", err);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Database operation failed"
                }))
            }
            AppError::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": msg
            })),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Bad request",
                "message": msg
            })),
            AppError::Internal(msg) => {
                log::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": msg
                }))
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;