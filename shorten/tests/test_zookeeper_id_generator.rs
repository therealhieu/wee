mod utils;

use pretty_assertions::assert_eq;
use std::env;
use tracing::debug;

use wee_shorten::{
    app_config::AppConfig, outbound::zookeeper::id_generator::ZooKeeperIdGenerator,
    services::shorten_service::id_generator::IdGenerator,
};

use utils::init_tracing;

async fn set_up() -> i64 {
    init_tracing();
    env::set_var("RUN_MODE", "test");
    let current_ts = chrono::Utc::now().timestamp();
    env::set_var(
        "SHORTEN__ZOOKEEPER__ID_GENERATOR__SHARD_INFO__BASE_PATH",
        format!("wee-test-{}", current_ts),
    );

    current_ts
}

fn delete_recursive<'a>(
    zk: &'a ZooKeeperIdGenerator,
    path: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        let children = zk.client.get_children(path).await.unwrap();

        if let Some(children) = children {
            for child in children {
                let child_path = format!("{}/{}", path, child);
                delete_recursive(zk, &child_path).await;
            }
        }

        zk.client.delete(path, None).await.unwrap().unwrap();
        debug!("Deleted path: {}", path);
    })
}

async fn tear_down(ts: i64, zk: &ZooKeeperIdGenerator) {
    delete_recursive(zk, &format!("/wee-test-{}", ts)).await;
}

#[tokio::test]
async fn test_generate_id() {
    let ts = set_up().await;
    let config = AppConfig::load();
    let zk = ZooKeeperIdGenerator::new(config.zookeeper.clone())
        .await
        .unwrap();
    let id = zk.generate_id().await.unwrap();
    assert_eq!(id.parse::<i64>().unwrap(), 0);
    tear_down(ts, &zk).await;
}
