use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use wee_core::outbound::mongodb::url_repo::MongoUrlRepo;
use wee_redirect::{
    app_config::AppConfig, inbound::rest::handlers::redirect::redirect,
    outbound::redis::redirect_service_cache::RedisRedirectServiceCache,
    services::redirect_service::RedirectService,
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
    let mongo_url_repo = MongoUrlRepo::new(config.mongodb.clone()).await.unwrap();
    mongo_url_repo.ensure_indexes().await.unwrap();

    let redis_redirect_service_cache = RedisRedirectServiceCache::new(config.redis.clone())
        .await
        .unwrap();

    let redirect_service = Arc::new(RedirectService::new(
        redis_redirect_service_cache,
        mongo_url_repo,
    ));

    let router = Router::new()
        .route("/ping", get(|| async { "Pong!" }))
        .route("/{code}", get(redirect))
        .with_state(redirect_service)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
        .await
        .unwrap();

    info!("Listening on {}:{}", config.app.host, config.app.port);
    axum::serve(listener, router).await.unwrap();
}
