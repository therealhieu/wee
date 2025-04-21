use std::future::Future;

use mongodb::bson::Document;

use crate::domain::entities::url::Url;

nest! {
    #[derive(Debug, thiserror::Error)]*
    pub enum UrlRepoError {
        #[error("Get URL error: {0}")]
        Get(#[from] pub enum GetUrlError {
            #[error("URL not found")]
            NotFound,

            #[error("Client Error: {0}")]
            ClientError(anyhow::Error),

            #[error("Internal Errorr: {0}")]
            InternalError(anyhow::Error),
        }),
        #[error("Insert URL error: {0}")]
        Insert(#[from] pub enum InsertUrlError {
            #[error("URL already exists")]
            AlreadyExists,

            #[error("Invalid URL")]
            InvalidUrl,

            #[error("Client Error: {0}")]
            ClientError(anyhow::Error),
        }),
        #[error("Replace URL error: {0}")]
        Replace(#[from] pub enum ReplaceUrlError {
            #[error("Client Error: {0}")]
            ClientError(anyhow::Error),

            #[error("URL not found: {0}")]
            NotFound(String),
        }),
    }
}

pub trait UrlRepo: Send + Sync {
    type InsertOutput: std::fmt::Debug + Send + Sync;

    fn get(&self, _short: &str) -> impl Future<Output = Result<Url, UrlRepoError>> + Send;
    fn insert(
        &self,
        _url: Url,
    ) -> impl Future<Output = Result<Self::InsertOutput, UrlRepoError>> + Send;
    fn replace_if_exists(&self, _url: Url)
    -> impl Future<Output = Result<(), UrlRepoError>> + Send;

    fn find<T>(&self, query: T) -> impl Future<Output = Result<Option<Url>, UrlRepoError>> + Send
    where
        T: Into<Document> + Send + Sync;
}
