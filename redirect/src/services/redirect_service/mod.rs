pub mod cache;
pub mod error;

use cache::RedirectServiceCache;
use error::RedirectServiceError;
use mongodb::bson::doc;
use wee_core::domain::repos::url_repo::UrlRepo;

pub trait RedirectServiceTrait: Send + Sync {
    fn redirect(
        &self,
        code: &str,
    ) -> impl Future<Output = Result<String, RedirectServiceError>> + Send;
}

pub struct RedirectService<C: RedirectServiceCache, R: UrlRepo> {
    pub cache: C,
    pub repository: R,
}

impl<C: RedirectServiceCache, R: UrlRepo> RedirectServiceTrait for RedirectService<C, R> {
    async fn redirect(&self, code: &str) -> Result<String, RedirectServiceError> {
        if let Some(url) = self.cache.get(code).await? {
            return Ok(url.long);
        }

        let url = self
            .repository
            .find(doc! {
                "$or": [
                    { "alias": code },
                    { "short": code },
                ]
            })
            .await?
            .ok_or(RedirectServiceError::UrlNotFound(code.to_string()))?;

        self.cache.set(url.clone()).await?;

        Ok(url.long)
    }
}

impl<C: RedirectServiceCache, R: UrlRepo> RedirectService<C, R> {
    pub fn new(cache: C, repository: R) -> Self {
        Self { cache, repository }
    }
}
