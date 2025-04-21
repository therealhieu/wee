use std::future::Future;

use super::error::ShortenServiceError;

pub trait IdGenerator: Send + Sync {
    fn generate_id(&self) -> impl Future<Output = Result<String, ShortenServiceError>> + Send;
}
