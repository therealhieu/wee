mod utils;

use std::env;

use redis::AsyncCommands;
use utils::init_tracing;
use wee_core::domain::entities::url::Url;
use wee_shorten::{
    app_config::AppConfig, outbound::redis::shorten_service_cache::RedisShortenServiceCache,
    services::shorten_service::cache::ShortenServiceCache,
};

fn set_up() {
    init_tracing();
    env::set_var("RUN_MODE", "test");
}

async fn tear_down(cache: &mut RedisShortenServiceCache) {
    let _: () = cache.conn.lock().await.flushdb().await.unwrap();
    // assert that the cache is empty
    let keys: Vec<String> = cache.conn.lock().await.keys("*").await.unwrap();
    assert!(keys.is_empty());
}

#[tokio::test]
async fn test_cache_and_get_by_long_url() {
    set_up();
    let config = AppConfig::load();
    let mut cache = RedisShortenServiceCache::new(config.redis.clone())
        .await
        .unwrap();

    let url = Url::builder()
        .long("https://example.com".to_string())
        .short("https://wee.rs/abc123".to_string())
        .alias(Some("abc123".to_string()))
        .expiration_date(None)
        .user_id("test_user".to_string())
        .created_at(chrono::Utc::now().naive_utc())
        .updated_at(chrono::Utc::now().naive_utc())
        .build();

    // Test caching a URL
    cache.cache(&url).await.unwrap();

    // Test retrieving the cached URL
    let cached_url = cache
        .get_by_long_url("https://example.com", "test_user")
        .await
        .unwrap();

    assert!(cached_url.is_some());
    let cached_url = cached_url.unwrap();
    assert_eq!(cached_url, url);

    // Test retrieving non-existent URL
    let non_existent = cache
        .get_by_long_url("https://nonexistent.com", "test_user")
        .await
        .unwrap();
    assert!(non_existent.is_none());

    tear_down(&mut cache).await;
}
