use wee_core::outbound::{mongodb::MongoConfig, redis::RedisConfig};

use crate::outbound::zookeeper::ZooKeeperConfig;

nest! {
    #[derive(Debug, thiserror::Error)]*
    pub enum AppConfigError {
        #[error("App Not Ready")]
        AppNotReady,
    }
}

nest! {
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Builder)]*
    #[serde(rename_all = "snake_case", deny_unknown_fields)]
    pub struct AppConfig {
        pub app: pub struct AppInfo {
            #[builder(into, default = "https://wee.rs")]
            pub host: String,
            pub port: u16,
        },
        pub mongodb: MongoConfig,
        pub zookeeper: ZooKeeperConfig,
        pub redis: RedisConfig,
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "shorten".to_string());
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "dev".to_string());

        let config = config::Config::builder()
            .add_source(config::File::with_name("configs/default"))
            .add_source(config::File::with_name(&format!("configs/{}", run_mode)).required(false))
            .add_source(
                config::Environment::with_prefix(&app_name.to_uppercase())
                    .try_parsing(true)
                    .separator("__"),
            )
            .build()
            .expect("Failed to load configuration");

        config
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize configuration")
    }
}

#[cfg(test)]
mod tests {
    use map_macro::hash_map;
    use pretty_assertions::assert_eq;
    use std::env;
    use wee_core::outbound::{mongodb::MongoConfig, redis::RedisConfig};

    use crate::outbound::zookeeper::id_generator::{ShardInfo, ZooKeeperIdGeneratorConfig};

    use super::*;

    #[test]
    fn test_load_config() {
        let config = AppConfig::load();
        let default = AppConfig::builder()
            .app(AppInfo::builder().host("localhost").port(3000).build())
            .mongodb(
                MongoConfig::builder()
                    .host("localhost")
                    .port(27017)
                    .username("test")
                    .password("test")
                    .database("wee")
                    .collections(hash_map! {
                        "url_repo".to_string() => "urls".to_string(),
                    })
                    .build(),
            )
            .zookeeper(
                ZooKeeperConfig::builder()
                    .host("0.0.0.0")
                    .port(2181)
                    .id_generator(
                        ZooKeeperIdGeneratorConfig::builder()
                            .shard_info(
                                ShardInfo::builder()
                                    .base_path("wee")
                                    .id(0)
                                    .start(0)
                                    .end(1_000)
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .redis(
                RedisConfig::builder()
                    .host("localhost")
                    .port(6379)
                    .dbs(hash_map! {
                        "shorten".to_string() => 0,
                    })
                    .build(),
            )
            .build();
        assert_eq!(config, default);

        env::set_var("SHORTEN__ZOOKEEPER__HOST", "0.0.0.0");
        let config = AppConfig::load();
        assert_eq!(config.zookeeper.host, "0.0.0.0");

        env::set_var("RUN_MODE", "test");
        let config = AppConfig::load();
        assert_eq!(
            config.zookeeper.id_generator.shard_info.base_path,
            "wee-test"
        );
    }
}
