use bson::bson;
use futures_util::TryStreamExt;
use mongodb::{
    Collection, IndexModel,
    bson::{self, Document, doc},
    error::{ErrorKind, WriteFailure},
    options::IndexOptions,
    results::InsertOneResult,
};
use tap::Tap;
use tracing::{debug, error, info, instrument, warn};

use crate::domain::{
    entities::url::Url,
    metadata::url_indexes::{UrlIndex, UrlIndexes},
    repos::url_repo::{GetUrlError, InsertUrlError, ReplaceUrlError, UrlRepo, UrlRepoError},
};

use super::MongoConfig;

#[derive(Debug, thiserror::Error)]
pub enum MongoUrlRepoError {
    #[error("MongoDB Client Error: {0}")]
    ClientError(#[from] mongodb::error::Error),
}

nest! {
    #[derive(Debug)]*
    pub struct MongoUrlRepo {
        pub config: MongoConfig,
        pub collection: Collection<Url>,

    }
}

impl UrlRepo for MongoUrlRepo {
    type InsertOutput = InsertOneResult;

    #[instrument(skip(self), fields(short = %short))]
    async fn get(&self, short: &str) -> Result<Url, UrlRepoError> {
        match self.collection.find_one(doc! {"short": short}).await {
            Ok(Some(url)) => {
                debug!("URL found: {}", url.short);
                Ok(url)
            }
            Ok(None) => {
                warn!("URL not found: {}", short);
                Err(UrlRepoError::Get(GetUrlError::NotFound))
            }
            Err(e) => {
                error!("MongoDB find_one error: {:?}", e);
                Err(UrlRepoError::Get(GetUrlError::ClientError(e.into())))
            }
        }
    }

    #[instrument(skip(self), fields(url = %url.long))]
    async fn insert(&self, url: Url) -> Result<Self::InsertOutput, UrlRepoError> {
        match self.collection.insert_one(url).await {
            Ok(result) => {
                debug!("URL inserted successfully");
                Ok(result)
            }
            Err(err) => {
                if let ErrorKind::Write(write_failure) = err.kind.as_ref() {
                    match write_failure {
                        WriteFailure::WriteError(write_error) if write_error.code == 11000 => {
                            warn!("Duplicate key error: URL already exists: {}", err);
                            return Err(UrlRepoError::Insert(InsertUrlError::AlreadyExists));
                        }
                        _ => {}
                    }
                }

                error!("MongoDB insert error: {:?}", err);
                Err(UrlRepoError::Insert(InsertUrlError::ClientError(
                    err.into(),
                )))
            }
        }
    }

    #[instrument(skip(self), fields(url = %url.short))]
    async fn replace_if_exists(&self, url: Url) -> Result<(), UrlRepoError> {
        self.collection
            .find_one_and_replace(doc! {"short": url.short.clone()}, url.clone())
            .await
            .map_err(|err| UrlRepoError::Replace(ReplaceUrlError::ClientError(err.into())))?
            .ok_or(UrlRepoError::Replace(ReplaceUrlError::NotFound(url.short)))?;

        Ok(())
    }

    async fn find<T>(&self, query: T) -> Result<Option<Url>, UrlRepoError>
    where
        T: Into<Document> + Send + Sync,
    {
        let query: Document = query.into();

        match self.collection.find_one(query.clone()).await {
            Ok(Some(url)) => Ok(Some(url)),
            Ok(None) => Err(UrlRepoError::Get(GetUrlError::NotFound)),
            Err(e) => Err(UrlRepoError::Get(GetUrlError::ClientError(e.into()))),
        }
    }
}

pub trait IntoIndexModel {
    fn into_index_model(self) -> IndexModel;
}

impl IntoIndexModel for UrlIndex {
    fn into_index_model(self) -> IndexModel {
        let keys = Document::from_iter(self.keys.iter().map(|key| (key.clone(), bson!(1))));
        let options = IndexOptions::builder()
            .unique(Some(self.is_unique))
            .sparse(Some(self.is_sparse))
            .build();

        IndexModel::builder()
            .keys(keys)
            .options(Some(options))
            .build()
    }
}

impl MongoUrlRepo {
    #[instrument(skip(config))]
    pub async fn new(config: MongoConfig) -> Result<Self, MongoUrlRepoError> {
        let client = mongodb::Client::with_uri_str(&config.uri()).await?;
        let db = client
            .database(&config.database)
            .tap(|db| info!("Connected to database: {}", db.name()));

        info!(
            "Creating collection if not exists: {}",
            &config.collections["url_repo"]
        );
        db.create_collection(&config.collections["url_repo"])
            .await?;

        let collection = db
            .collection::<Url>(&config.collections["url_repo"])
            .tap(|collection| info!("Connected to collection: {}", collection.name()));

        Ok(MongoUrlRepo { config, collection })
    }

    #[instrument(skip(self))]
    pub async fn ensure_indexes(&self) -> Result<(), MongoUrlRepoError> {
        let url_indexes = UrlIndexes::default();
        info!(
            "Ensuring indexes for collection: {}",
            self.collection.name()
        );
        info!("Indexes: {:#?}", url_indexes);

        let mongo_existing_indexes = self
            .collection
            .list_indexes()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        info!("Mongo existing indexes: {:#?}", mongo_existing_indexes);

        for new in url_indexes.values {
            // Find matching existing index by key
            let new_keys = Document::from_iter(new.keys.iter().map(|key| (key.clone(), bson!(1))));

            let matched = mongo_existing_indexes
                .iter()
                .find(|existing| existing.keys == new_keys);

            if let Some(existing) = matched {
                if let Some(existing_opts) = existing.options.as_ref() {
                    let same_unique = existing_opts.unique.unwrap_or_default() == new.is_unique;
                    let same_sparse = existing_opts.sparse.unwrap_or_default() == new.is_sparse;

                    if same_unique && same_sparse {
                        continue;
                    }

                    info!(
                        "Dropping index: {}",
                        existing_opts
                            .name
                            .as_ref()
                            .unwrap_or(&"unknown_index_name".to_string())
                    );
                    self.collection
                        .drop_index(
                            existing_opts
                                .name
                                .as_ref()
                                .unwrap_or(&"unknown_index_name".to_string()),
                        )
                        .await?;
                }
            }

            let mongo_index = new.into_index_model();
            info!("Creating index: {:#?}", mongo_index);
            self.collection.create_index(mongo_index).await?;
        }

        Ok(())
    }
}
