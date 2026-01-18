use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AssetQuery {
    pub folder_id: Option<uuid::Uuid>,
}
