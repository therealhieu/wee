use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "snake_case")]
pub struct RedisConfig {
    #[builder(into)]
    pub host: String,
    pub port: u16,
    pub dbs: HashMap<String, u8>,
}
