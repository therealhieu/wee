use std::future::Future;

use wee_core::domain::entities::url::Url;

use super::error::ShortenServiceError;

pub trait ShortenServiceCache: Send + Sync {
    fn get_by_long_url(
        &self,
        long_url: &str,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<Url>, ShortenServiceError>> + Send;

    fn get_by_alias(
        &self,
        alias: &str,
    ) -> impl Future<Output = Result<Option<Url>, ShortenServiceError>> + Send;

    fn cache(&self, url: &Url) -> impl Future<Output = Result<(), ShortenServiceError>> + Send;
}
