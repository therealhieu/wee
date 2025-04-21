use chrono::{NaiveDate, NaiveDateTime, Utc};

use super::Entity;

#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub long: String,
    pub short: String,
    #[builder(required, into)]
    pub alias: Option<String>,
    #[builder(required, into)]
    pub expiration_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_id: String,
}

impl Url {
    pub fn expired(&self) -> bool {
        self.expiration_date.is_some() && self.expiration_date.unwrap() < Utc::now().date_naive()
    }
}

impl Entity for Url {}
