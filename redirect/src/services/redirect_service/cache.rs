use wee_core::domain::entities::url::Url;

use super::error::RedirectServiceError;

pub trait RedirectServiceCache: Send + Sync {
    fn get(
        &self,
        code: &str,
    ) -> impl Future<Output = Result<Option<Url>, RedirectServiceError>> + Send;

    fn set(&self, url: Url) -> impl Future<Output = Result<(), RedirectServiceError>> + Send;
}
