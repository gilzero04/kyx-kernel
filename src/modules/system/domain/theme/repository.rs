use super::entity::Theme;
use crate::core::AppResult;
use crate::modules::system::interface::http::dto::theme::{CreateThemeDto, UpdateThemeDto};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ThemeRepository: Send + Sync {
    async fn create(&self, dto: CreateThemeDto) -> AppResult<Theme>;
    #[allow(dead_code)]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Theme>>;

    // Finds themes visible to a specific tenant (Public + Own + Parent/Restricted if applicable)
    // If tenant_id is None, lists ALL themes (for Platform Admin)
    async fn find_available(&self, tenant_id: Option<Uuid>) -> AppResult<Vec<Theme>>;

    #[allow(dead_code)]
    async fn update(&self, id: Uuid, dto: UpdateThemeDto) -> AppResult<Theme>;

    // Hard delete as per policy
    #[allow(dead_code)]
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    // Set active theme for a tenant
    #[allow(dead_code)]
    async fn set_active_theme(&self, tenant_id: Uuid, theme_id: Uuid) -> AppResult<()>;

    // Get active theme for a tenant
    #[allow(dead_code)]
    async fn get_active_theme(&self, tenant_id: Uuid) -> AppResult<Option<Theme>>;

    // Check if theme is currently in use (by any tenant or system)
    // Returns Some(reason) if in use, None if not.
    // exclude_tenant_id: Optional Tenant ID to ignore (e.g., the owner).
    async fn is_in_use(
        &self,
        theme_id: Uuid,
        exclude_tenant_id: Option<Uuid>,
    ) -> AppResult<Option<String>>;

    // Helper to find System Owner ID
    async fn get_owner_id(&self) -> AppResult<Option<Uuid>>;
}
