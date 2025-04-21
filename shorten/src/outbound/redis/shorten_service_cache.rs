use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use std::sync::Arc;
use tokio::sync::Mutex;
use wee_core::{
    domain::entities::{url::Url, Entity},
    outbound::redis::RedisConfig,
};

use crate::services::shorten_service::{cache::ShortenServiceCache, error::ShortenServiceError};

nest! {
    #[derive(Debug, thiserror::Error)]*
    pub enum RedisShortenServiceCacheError {
        #[error("Redis Client Error: {0}")]
        RedisClientError(#[from] redis::RedisError),

        #[error("Internal Error: {0}")]
        InternalError(#[from] anyhow::Error),
    }
}

nest! {
    #[derive(Debug)]*
    pub struct RedisShortenServiceCache {
        pub config: RedisConfig,
        pub client: Client,
        pub conn: Arc<Mutex<MultiplexedConnection>>,
    }
}

impl ShortenServiceCache for RedisShortenServiceCache {
    #[instrument(skip(self))]
    async fn get_by_long_url(
        &self,
        long_url: &str,
        user_id: &str,
    ) -> Result<Option<Url>, ShortenServiceError> {
        let key = format!("user:{}:urls", user_id);
        let mut conn = self.conn.lock().await;
        let value: Option<String> = conn
            .hget(key, long_url)
            .await
            .map_err(RedisShortenServiceCacheError::RedisClientError)?;

        if let Some(json) = value {
            let url =
                Url::from_json(&json).map_err(RedisShortenServiceCacheError::InternalError)?;
            debug!("Retrieved URL: {}", url.long);

            Ok(Some(url))
        } else {
            debug!(
                "No URL found in cache for long_url {} and user_id {}",
                long_url, user_id
            );
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn get_by_alias(&self, alias: &str) -> Result<Option<Url>, ShortenServiceError> {
        let key = format!("alias:{}", alias);
        let mut conn = self.conn.lock().await;
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(RedisShortenServiceCacheError::RedisClientError)?;

        if let Some(json) = value {
            let url =
                Url::from_json(&json).map_err(RedisShortenServiceCacheError::InternalError)?;
            debug!("Retrieved URL: {}", url.long);

            Ok(Some(url))
        } else {
            debug!("No URL found in cache for alias {}", alias);
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn cache(&self, url: &Url) -> Result<(), ShortenServiceError> {
        let key = format!("user:{}:urls", url.user_id);
        let value = url
            .to_json()
            .map_err(RedisShortenServiceCacheError::InternalError)?;

        let mut conn = self.conn.lock().await;
        let _: () = conn
            .hset(key, &url.long, &value)
            .await
            .map_err(RedisShortenServiceCacheError::RedisClientError)?;

        if let Some(alias) = url.alias.as_ref() {
            let alias_key = format!("alias:{}", alias);
            let _: () = conn
                .set(alias_key, &value)
                .await
                .map_err(RedisShortenServiceCacheError::RedisClientError)?;
        }

        let key = format!("short:{}", url.short);
        let _: () = conn
            .set(key, &value)
            .await
            .map_err(RedisShortenServiceCacheError::RedisClientError)?;

        debug!("Cached URL: {}", url.long);

        Ok(())
    }
}

impl RedisShortenServiceCache {
    pub async fn new(config: RedisConfig) -> Result<Self, RedisShortenServiceCacheError> {
        let client = Client::open(format!(
            "redis://{}:{}/{}",
            config.host, config.port, config.dbs["shorten"]
        ))?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(RedisShortenServiceCache {
            config,
            client,
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}
