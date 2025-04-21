pub mod url_repo;

use std::collections::HashMap;

nest! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]*
    pub struct MongoConfig {
        #[builder(into)]
        pub host: String,
        pub port: u16,
        #[builder(into)]
        pub database: String,
        #[builder(into)]
        pub username: String,
        #[builder(into)]
        pub password: String,
        pub collections: HashMap<String, String>,
    }
}

impl MongoConfig {
    pub fn uri(&self) -> String {
        format!(
            "mongodb://{}:{}@{}:{}/?authSource=admin",
            self.username, self.password, self.host, self.port
        )
    }
}
