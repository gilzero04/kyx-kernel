use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct LogsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub search: Option<String>,
}
