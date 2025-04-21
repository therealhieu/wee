use id_generator::ZooKeeperIdGeneratorConfig;

pub mod id_generator;

nest! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]*
    pub struct ZooKeeperConfig {
        #[builder(into)]
        pub host: String,
        pub port: u16,
        pub id_generator: ZooKeeperIdGeneratorConfig,
    }
}
