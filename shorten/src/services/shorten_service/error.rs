use wee_core::domain::repos::url_repo::UrlRepoError;

use crate::outbound::{
    redis::shorten_service_cache::RedisShortenServiceCacheError,
    zookeeper::id_generator::ZooKeeperIdGeneratorError,
};

#[derive(Debug, thiserror::Error)]
pub enum ShortenServiceError {
    #[error("UrlRepoError: {0}")]
    UrlRepoError(#[from] UrlRepoError),

    #[error("Generate Id Error: {0}")]
    IdGeneratorError(#[from] ZooKeeperIdGeneratorError),

    #[error("Cache Error: {0}")]
    CacheError(#[from] RedisShortenServiceCacheError),

    #[error("Internal Error: {0}")]
    InternalError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Alias already taken: {0}")]
    AliasTaken(String),

    #[error("Url already existed with alias: {0}")]
    UrlAlreadyExistedWithAlias(String),
}
