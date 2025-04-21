use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{inbound::rest::error::ApiError, services::redirect_service::RedirectServiceTrait};

pub async fn redirect<S>(
    State(redirect_service): State<Arc<S>>,
    Path(code): Path<String>,
) -> Result<Redirect, ApiError>
where
    S: RedirectServiceTrait,
{
    let url = redirect_service.redirect(&code).await?;

    Ok(Redirect::temporary(url.as_str()))
}
