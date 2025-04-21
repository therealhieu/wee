use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

use wee_core::outbound::mongodb::url_repo::MongoUrlRepo;
use wee_shorten::{
    app_config::AppConfig,
    inbound::rest::handlers::shorten::shorten,
    outbound::{
        redis::shorten_service_cache::RedisShortenServiceCache,
        zookeeper::id_generator::ZooKeeperIdGenerator,
    },
    services::shorten_service::ShortenService,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    info!("Starting the application...");
    let config = AppConfig::load();
    info!("Config: {:#?}", config);

    let zk_id_generator = ZooKeeperIdGenerator::new(config.zookeeper.clone())
        .await
        .unwrap();

    let mongo_url_repo = MongoUrlRepo::new(config.mongodb.clone()).await.unwrap();
    mongo_url_repo.ensure_indexes().await.unwrap();

    let redis_shorten_service_cache = RedisShortenServiceCache::new(config.redis.clone())
        .await
        .unwrap();
    let shorten_service = Arc::new(ShortenService::new(
        zk_id_generator,
        mongo_url_repo,
        redis_shorten_service_cache,
    ));

    let router = Router::new()
        .route("/ping", get(|| async { "Pong!" }))
        .route("/urls", post(shorten))
        .with_state(shorten_service)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
    let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
        .await
        .unwrap();

    info!("Listening on {}:{}", config.app.host, config.app.port);

    axum::serve(listener, router).await.unwrap();
}
