use axum::{http::StatusCode, response::IntoResponse};
use validator::ValidationErrors;

use crate::services::shorten_service::error::ShortenServiceError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not Found")]
    NotFound,
    #[error("Bad Request")]
    BadRequest,
    #[error("Validation Error: {0}")]
    ValidationError(String),
    #[error("Shorten Service Error: {0}")]
    ShortenServiceError(#[from] ShortenServiceError),
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        let message = errors
            .errors()
            .iter()
            .map(|(field, error)| format!("Field: {}, Error: {:?}", field, error))
            .collect::<Vec<_>>()
            .join(", ");
        ApiError::ValidationError(message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            ApiError::BadRequest => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            ApiError::ValidationError(message) => {
                (StatusCode::BAD_REQUEST, message).into_response()
            }
            ApiError::ShortenServiceError(error) => match error {
                ShortenServiceError::AliasTaken(_) => {
                    (StatusCode::BAD_REQUEST, error.to_string()).into_response()
                }
                ShortenServiceError::UrlAlreadyExistedWithAlias(_) => {
                    (StatusCode::BAD_REQUEST, error.to_string()).into_response()
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
            },
        }
    }
}
