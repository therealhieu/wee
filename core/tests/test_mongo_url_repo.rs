mod utils;

use futures_util::TryStreamExt;
use map_macro::hash_map;
use mongodb::Client;
use tracing::{debug, error};
use utils::init_tracing;
use wee_core::outbound::mongodb::{MongoConfig, url_repo::MongoUrlRepo};

async fn set_up() -> MongoUrlRepo {
    init_tracing();

    let config = MongoConfig::builder()
        .host("localhost")
        .port(27017)
        .database("test")
        .username("test")
        .password("test")
        .collections(hash_map! {
            "url_repo".to_string() => "collection-test-{}".to_string(),
        })
        .build();

    // ensure the database is created
    let uri = config.uri();
    debug!("Connecting to database: {}", uri);
    let client = Client::with_uri_str(&uri).await.unwrap();
    let create_collection = client
        .database(&config.database)
        .create_collection(&config.collections["url_repo"])
        .await;

    match create_collection {
        Ok(_) => (),
        Err(e) => {
            error!("Error creating database: {}", e);
        }
    }

    MongoUrlRepo::new(config.clone()).await.unwrap()
}

async fn tear_down(mongo_url_repo: MongoUrlRepo) {
    debug!("Tearing down test environment");
    mongo_url_repo.collection.drop().await.unwrap();
    debug!("Dropped collection: {}", mongo_url_repo.collection.name());
}

#[tokio::test]
async fn test_ensure_indexes() {
    let mongo_url_repo = set_up().await;
    mongo_url_repo.ensure_indexes().await.unwrap();

    let existing_indexes = mongo_url_repo
        .collection
        .list_indexes()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    debug!("Existing indexes after test: {:#?}", existing_indexes);

    tear_down(mongo_url_repo).await;
}
