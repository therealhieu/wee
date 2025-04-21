use wee_core::outbound::{mongodb::MongoConfig, redis::RedisConfig};

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

    use super::*;

    #[test]
    fn test_load_config() {
        let config = AppConfig::load();
        let default = AppConfig::builder()
            .app(AppInfo::builder().host("localhost").port(3001).build())
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
            .redis(
                RedisConfig::builder()
                    .host("localhost")
                    .port(6379)
                    .dbs(hash_map! {
                        "redirect".to_string() => 0,
                    })
                    .build(),
            )
            .build();
        assert_eq!(config, default);

        unsafe { std::env::set_var("RUN_MODE", "test") };
        let config = AppConfig::load();
        assert_eq!(config.mongodb.collections["url_repo"], "urls-test");
    }
}
