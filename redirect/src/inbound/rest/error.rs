use axum::{http::StatusCode, response::IntoResponse};
use wee_core::domain::repos::url_repo::{GetUrlError, UrlRepoError};

use crate::services::redirect_service::error::RedirectServiceError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Redirect Service Error: {0}")]
    RedirectServiceError(#[from] RedirectServiceError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::RedirectServiceError(err) => match err {
                RedirectServiceError::UrlNotFound(_) => {
                    (StatusCode::NOT_FOUND, err.to_string()).into_response()
                }
                RedirectServiceError::UrlRepoError(err) => match err {
                    UrlRepoError::Get(GetUrlError::NotFound) => {
                        (StatusCode::NOT_FOUND, err.to_string()).into_response()
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
                },
                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            },
        }
    }
}
