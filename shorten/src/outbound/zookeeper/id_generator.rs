use std::net::ToSocketAddrs;

use tokio_zookeeper::{self as tzk, Acl, CreateMode};

use crate::services::shorten_service::{error::ShortenServiceError, id_generator::IdGenerator};

use super::ZooKeeperConfig;

nest! {
    #[derive(Debug, thiserror::Error)]*
    pub enum ZooKeeperIdGeneratorError {
        #[error("ZooKeeperIdGenerator Client Error: {0}")]
        ClientError(#[from] tzk::error::Error),

        #[error("ZooKeeperIdGenerator Create Error: {0}")]
        CreateError(#[from] tzk::error::Create),

        #[error("Id Not Found")]
        IdNotFound,

        #[error("Internal Error: {0}")]
        InternalError(#[from] anyhow::Error),
    }
}

nest! {
    pub struct ZooKeeperIdGenerator {
        pub config:
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]*
            pub struct ZooKeeperIdGeneratorConfig {
                pub shard_info:
                    pub struct ShardInfo {
                        #[builder(into)]
                        pub base_path: String,
                        pub id: usize,
                        pub start: usize,
                        pub end: usize,
                    },
            },
        pub client: tzk::ZooKeeper,
    }

}

impl ShardInfo {
    pub fn id_path(&self) -> String {
        format!(
            "/{}/shard-{}-{}-{}/",
            self.base_path, self.id, self.start, self.end
        )
    }
}

impl IdGenerator for ZooKeeperIdGenerator {
    #[instrument(skip(self))]
    async fn generate_id(&self) -> Result<String, ShortenServiceError> {
        let path = self
            .client
            .create(
                &self.config.shard_info.id_path(),
                b"",
                Acl::open_unsafe(),
                CreateMode::PersistentSequential,
            )
            .await
            .map_err(ZooKeeperIdGeneratorError::ClientError)?
            .map_err(ZooKeeperIdGeneratorError::CreateError)?;

        tracing::debug!("Created sequential node: {}", path);

        let id = path
            .split('/')
            .next_back()
            .ok_or_else(|| {
                ZooKeeperIdGeneratorError::InternalError(anyhow::anyhow!(
                    "Failed to parse ID from path: {}",
                    path
                ))
            })?
            .to_string()
            .parse::<usize>()
            .map_err(|err| {
                ZooKeeperIdGeneratorError::InternalError(anyhow::anyhow!(
                    "Failed to parse ID to usize: {}",
                    err
                ))
            })?;

        let global_id = self.config.shard_info.start + id;

        // If global_id is greater than self.config.shard_info.end, gracefully shutdown the service
        if global_id > self.config.shard_info.end {
            info!("Global ID is greater than the end of the shard");
            std::process::exit(1);
        }

        Ok(global_id.to_string())
    }
}

impl ZooKeeperIdGenerator {
    pub async fn new(config: ZooKeeperConfig) -> Result<Self, ZooKeeperIdGeneratorError> {
        let address = (config.host, config.port)
            .to_socket_addrs()
            .map_err(|err| {
                ZooKeeperIdGeneratorError::InternalError(anyhow::anyhow!(
                    "Failed to parse address: {}",
                    err
                ))
            })?
            .next()
            .ok_or_else(|| {
                ZooKeeperIdGeneratorError::InternalError(anyhow::anyhow!("Failed to parse address"))
            })?;
        info!("Address: {}", address);

        let (client, _watcher) = tzk::ZooKeeper::connect(&address).await?;

        let zk = ZooKeeperIdGenerator {
            config: config.id_generator.clone(),
            client,
        };
        zk.ensure_shard_path_exist().await?;

        Ok(zk)
    }

    #[instrument(skip(self))]
    pub async fn ensure_shard_path_exist(&self) -> Result<(), ZooKeeperIdGeneratorError> {
        let nodes = format!(
            "{}/shard-{}-{}-{}",
            self.config.shard_info.base_path,
            self.config.shard_info.id,
            self.config.shard_info.start,
            self.config.shard_info.end
        )
        .split("/")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        let mut path = String::new();

        for node in nodes {
            path = format!("{}/{}", path, node);
            if self.client.watch().exists(&path).await?.is_none() {
                // Skip if the path already exists
                let create = self
                    .client
                    .create(&path, b"", Acl::open_unsafe(), CreateMode::Persistent)
                    .await?;

                match create {
                    Ok(path) => info!("Created path: {}", path),
                    Err(create_err) => match create_err {
                        tzk::error::Create::NodeExists => info!("Path already exists: {}", path),
                        _ => return Err(ZooKeeperIdGeneratorError::CreateError(create_err)),
                    },
                }
            }
        }

        Ok(())
    }
}
