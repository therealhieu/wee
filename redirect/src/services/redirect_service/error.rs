use wee_core::domain::repos::url_repo::UrlRepoError;

use crate::outbound::redis::redirect_service_cache::RedisRedirectServiceCacheError;

#[derive(Debug, thiserror::Error)]
pub enum RedirectServiceError {
    #[error("Url Repo Error: {0}")]
    UrlRepoError(#[from] UrlRepoError),

    #[error("Url Not Found: {0}")]
    UrlNotFound(String),

    #[error("Cache Error: {0}")]
    CacheError(#[from] RedisRedirectServiceCacheError),
}
