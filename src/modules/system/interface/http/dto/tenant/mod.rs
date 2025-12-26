use serde::Deserialize;
use utoipa::{ToSchema, IntoParams};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TenantsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOwnerRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
}
