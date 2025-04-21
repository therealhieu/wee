pub mod cache;
pub mod error;
pub mod id_generator;

use std::future::Future;
use std::sync::Arc;

use cache::ShortenServiceCache;
use chrono::NaiveDate;
use error::ShortenServiceError;
use id_generator::IdGenerator;
use tap::Pipe;
use tracing::debug;
use wee_core::domain::entities::url::Url;
use wee_core::domain::repos::url_repo::UrlRepo;

#[derive(Debug, Clone, Builder)]
pub struct ShortenParams {
    pub url: String,
    pub user_id: String,
    pub alias: Option<String>,
    pub expiration_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ShortenResult {
    pub short: String,
    #[builder(required, into)]
    pub alias: Option<String>,
    #[builder(required, into)]
    pub expiration_date: Option<NaiveDate>,
}

pub trait ShortenServiceTrait: Send + Sync {
    fn shorten(
        &self,
        params: ShortenParams,
    ) -> impl Future<Output = Result<ShortenResult, ShortenServiceError>> + Send;
}

#[derive(Debug, Clone)]
pub struct ShortenService<G: IdGenerator, R: UrlRepo, C: ShortenServiceCache> {
    pub id_generator: Arc<G>,
    pub repository: Arc<R>,
    pub cache: Arc<C>,
}

impl<G: IdGenerator, R: UrlRepo, C: ShortenServiceCache> ShortenServiceTrait
    for ShortenService<G, R, C>
{
    #[instrument(skip(self))]
    async fn shorten(&self, params: ShortenParams) -> Result<ShortenResult, ShortenServiceError> {
        if let Some(alias) = params.alias.as_ref() {
            if let Some(cached_url) = self.cache.get_by_alias(alias).await? {
                return self.process_when_alias_was_cached(params, cached_url).await;
            }
        }

        // Check if this long url already exists with current user_id
        if let Some(cached_url) = self
            .cache
            .get_by_long_url(&params.url, &params.user_id)
            .await?
        {
            return self
                .process_when_long_url_was_cached(params, cached_url)
                .await;
        }

        let url = self.generate_and_save_url(params).await?;

        Ok(ShortenResult::builder()
            .short(url.short)
            .alias(url.alias)
            .expiration_date(url.expiration_date)
            .build())
    }
}

impl<G: IdGenerator, R: UrlRepo, C: ShortenServiceCache> ShortenService<G, R, C> {
    pub fn new(id_generator: G, repository: R, cache: C) -> Self {
        ShortenService {
            id_generator: Arc::new(id_generator),
            repository: Arc::new(repository),
            cache: Arc::new(cache),
        }
    }

    pub async fn generate_url(
        &self,
        shorten_params: ShortenParams,
    ) -> Result<Url, ShortenServiceError> {
        let id_base62 = self
            .id_generator
            .generate_id()
            .await?
            .pipe(|id| {
                id.parse::<u128>()
                    .map_err(|err| ShortenServiceError::InternalError(err.into()))
            })?
            .pipe(base62::encode);

        let url = Url::builder()
            .long(shorten_params.url)
            .short(id_base62)
            .alias(shorten_params.alias)
            .expiration_date(shorten_params.expiration_date)
            .created_at(chrono::Utc::now().naive_utc())
            .updated_at(chrono::Utc::now().naive_utc())
            .user_id(shorten_params.user_id)
            .build();

        Ok(url)
    }

    pub async fn generate_and_save_url(
        &self,
        shorten_params: ShortenParams,
    ) -> Result<Url, ShortenServiceError> {
        let url = self.generate_url(shorten_params).await?;
        self.repository.insert(url.clone()).await?;
        self.cache.cache(&url).await?;

        Ok(url)
    }

    pub async fn process_when_alias_was_cached(
        &self,
        params: ShortenParams,
        cached_url: Url,
    ) -> Result<ShortenResult, ShortenServiceError> {
        // Check if this alias already exists in cache.
        // If yes, user_id of the cache must be the same as the current request.
        // If not, check if alias is available.
        if cached_url.expired() {
            let url = self.generate_url(params).await?;
            self.repository.replace_if_exists(url.clone()).await?;
            self.cache.cache(&url).await?;

            Ok(ShortenResult::builder()
                .short(url.short)
                .alias(url.alias)
                .expiration_date(url.expiration_date)
                .build())
        } else if params.alias != cached_url.alias {
            return Err(ShortenServiceError::UrlAlreadyExistedWithAlias(
                cached_url.alias.unwrap(),
            ));
        } else if params.alias.is_some() && params.url != cached_url.long {
            return Err(ShortenServiceError::AliasTaken(params.alias.unwrap()));
        } else {
            debug!(
                "Alias already exists with current user_id: {}",
                cached_url.short
            );
            Ok(ShortenResult::builder()
                .short(cached_url.short)
                .alias(cached_url.alias)
                .expiration_date(cached_url.expiration_date)
                .build())
        }
    }

    pub async fn process_when_long_url_was_cached(
        &self,
        params: ShortenParams,
        cached_url: Url,
    ) -> Result<ShortenResult, ShortenServiceError> {
        if cached_url.expired() {
            let url = self.generate_url(params).await?;
            self.repository.replace_if_exists(url.clone()).await?;
            self.cache.cache(&url).await?;

            Ok(ShortenResult::builder()
                .short(url.short)
                .alias(url.alias)
                .expiration_date(url.expiration_date)
                .build())
        } else {
            debug!(
                "Long url already exists with current user_id: {}",
                cached_url.short
            );
            Ok(ShortenResult::builder()
                .short(cached_url.short)
                .alias(cached_url.alias)
                .expiration_date(cached_url.expiration_date)
                .build())
        }
    }
}
