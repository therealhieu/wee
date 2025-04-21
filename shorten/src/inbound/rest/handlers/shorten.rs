use std::sync::Arc;

use axum::{extract::State, Json};
use chrono::{NaiveDate, Utc};
use validator::{Validate, ValidationError};

use crate::{
    inbound::rest::error::ApiError,
    services::shorten_service::{ShortenParams, ShortenResult, ShortenServiceTrait},
};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortenRequestPayload {
    #[validate(url)]
    pub url: String,
    pub user_id: String,
    pub alias: Option<String>,
    #[validate(custom(function = "validate_expiration_date"))]
    pub expiration_date: Option<NaiveDate>,
}

fn validate_expiration_date(date: &NaiveDate) -> Result<(), ValidationError> {
    let must_be_in_future = date > &Utc::now().date_naive();

    if must_be_in_future {
        Ok(())
    } else {
        Err(ValidationError::new("expiration_date_must_be_in_future"))
    }
}

impl From<ShortenRequestPayload> for ShortenParams {
    fn from(payload: ShortenRequestPayload) -> Self {
        ShortenParams {
            url: payload.url,
            user_id: payload.user_id,
            alias: payload.alias,
            expiration_date: payload.expiration_date,
        }
    }
}

pub async fn shorten<S>(
    State(shorten_service): State<Arc<S>>,
    Json(payload): Json<ShortenRequestPayload>,
) -> Result<Json<ShortenResult>, ApiError>
where
    S: ShortenServiceTrait,
{
    payload.validate()?;

    let shorten_result = shorten_service.shorten(payload.into()).await?;

    Ok(Json(shorten_result))
}
