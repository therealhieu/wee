use std::sync::Arc;

use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use tokio::sync::Mutex;
use tracing::debug;
use wee_core::{
    domain::entities::{Entity, url::Url},
    outbound::redis::RedisConfig,
};

use crate::services::redirect_service::{cache::RedirectServiceCache, error::RedirectServiceError};

#[derive(Debug, thiserror::Error)]
pub enum RedisRedirectServiceCacheError {
    #[error("Redis Client Error: {0}")]
    RedisClientError(#[from] redis::RedisError),

    #[error("Internal Error: {0}")]
    InternalError(#[from] anyhow::Error),
}

pub struct RedisRedirectServiceCache {
    pub client: Client,
    pub conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RedirectServiceCache for RedisRedirectServiceCache {
    #[instrument(skip(self))]
    async fn set(&self, url: Url) -> Result<(), RedirectServiceError> {
        let mut keys = vec![format!("short:{}", url.short)];

        if let Some(alias) = url.alias.as_ref() {
            keys.push(format!("alias:{}", alias));
        }

        let value = url
            .to_json()
            .map_err(RedisRedirectServiceCacheError::InternalError)?;
        let items = keys.iter().map(|key| (key, &value)).collect::<Vec<_>>();

        let () = self
            .conn
            .lock()
            .await
            .mset(&items)
            .await
            .map_err(RedisRedirectServiceCacheError::RedisClientError)?;

        debug!("Set url {} in cache with keys: {:?}", url.short, keys);

        Ok(())
    }

    async fn get(&self, code: &str) -> Result<Option<Url>, RedirectServiceError> {
        let keys: Vec<String> = vec![format!("short:{}", code), format!("alias:{}", code)];
        let mut conn = self.conn.lock().await;

        for key in keys {
            let value: Option<String> = conn
                .get(key)
                .await
                .map_err(RedisRedirectServiceCacheError::RedisClientError)?;

            if let Some(value) = value {
                return Ok(Some(
                    Url::from_json(&value)
                        .map_err(RedisRedirectServiceCacheError::InternalError)?,
                ));
            }
        }

        Ok(None)
    }
}

impl RedisRedirectServiceCache {
    pub async fn new(config: RedisConfig) -> Result<Self, RedisRedirectServiceCacheError> {
        let client = Client::open(format!("redis://{}:{}", config.host, config.port))?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}
