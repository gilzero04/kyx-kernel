use async_trait::async_trait;
use uuid::Uuid;
use crate::core::AppResult;
use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::theme::{ThemeRepository, Theme};
use crate::modules::system::interface::http::dto::theme::{CreateThemeDto, UpdateThemeDto};

pub struct PostgresThemeRepository {
    db: std::sync::Arc<Database>,
}

impl PostgresThemeRepository {
    pub fn new(db: std::sync::Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ThemeRepository for PostgresThemeRepository {
    async fn create(&self, dto: CreateThemeDto) -> AppResult<Theme> {
        let theme = sqlx::query_as::<_, Theme>(
            r#"
            INSERT INTO sys_themes (name, description, config, visibility, tenant_id, author, preview_url, logo_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(dto.name)
        .bind(dto.description)
        .bind(dto.config)
        .bind(dto.visibility)
        .bind(dto.tenant_id)
        .bind(dto.author)
        .bind(dto.preview_url)
        .bind(dto.logo_url)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(theme)
    }

    #[allow(dead_code)]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Theme>> {
        let theme = sqlx::query_as::<_, Theme>(
            "SELECT * FROM sys_themes WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(theme)
    }

    async fn find_available(&self, tenant_id: Option<Uuid>) -> AppResult<Vec<Theme>> {
        let themes = if let Some(tid) = tenant_id {
            // Find Public OR Private (Owned by Tenant)
            // TODO: Add 'Restricted' logic via sys_theme_access join
            sqlx::query_as::<_, Theme>(
                r#"
                SELECT * FROM sys_themes 
                WHERE 
                    visibility = 'public' 
                    OR tenant_id = $1
                    OR (
                        visibility = 'restricted' AND EXISTS (
                            SELECT 1 FROM sys_theme_access 
                            WHERE theme_id = sys_themes.id AND tenant_id = $1
                        )
                    )
                ORDER BY created_at DESC
                "#
            )
            .bind(tid)
            .fetch_all(&self.db.pool)
            .await?
        } else {
            // System Admin sees ALL
            sqlx::query_as::<_, Theme>(
                "SELECT * FROM sys_themes ORDER BY created_at DESC"
            )
            .fetch_all(&self.db.pool)
            .await?
        };

        Ok(themes)
    }

    #[allow(dead_code)]
    async fn update(&self, id: Uuid, dto: UpdateThemeDto) -> AppResult<Theme> {
        // Dynamic update builder would be better, but simple for now
        // Usually we fetch then update, or use COALESCE in SQL.
        let theme = sqlx::query_as::<_, Theme>(
            r#"
            UPDATE sys_themes
            SET 
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                config = COALESCE($4, config),
                visibility = COALESCE($5, visibility),
                is_active = COALESCE($6, is_active),
                author = COALESCE($7, author),
                preview_url = COALESCE($8, preview_url),
                logo_url = COALESCE($9, logo_url),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(dto.name)
        .bind(dto.description)
        .bind(dto.config)
        .bind(dto.visibility)
        .bind(dto.is_active)
        .bind(dto.author)
        .bind(dto.preview_url)
        .bind(dto.logo_url)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(theme)
    }

    #[allow(dead_code)]
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM sys_themes WHERE id = $1")
        .bind(id)
        .execute(&self.db.pool)
        .await?;
        
        Ok(())
    }

    #[allow(dead_code)]
    async fn set_active_theme(&self, tenant_id: Uuid, theme_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE auth_tenants SET active_theme_id = $2, updated_at = NOW() WHERE id = $1"
        )
        .bind(tenant_id)
        .bind(theme_id)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn get_active_theme(&self, tenant_id: Uuid) -> AppResult<Option<Theme>> {
        // Join tenant with theme
        let theme = sqlx::query_as::<_, Theme>(
            r#"
            SELECT t.* 
            FROM sys_themes t
            JOIN auth_tenants at ON at.active_theme_id = t.id
            WHERE at.id = $1
            "#
        )
        .bind(tenant_id)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(theme)
    }

    async fn is_in_use(&self, theme_id: Uuid, exclude_tenant_id: Option<Uuid>) -> AppResult<Option<String>> {
        let theme = self.find_by_id(theme_id).await?;
        if theme.is_none() {
            return Ok(None);
        }
        let theme_entity = theme.unwrap();
        let theme_name = theme_entity.name;
        let theme_tenant_id = theme_entity.tenant_id;

        // 2. Check Tenants (active_theme_id)
        // Return ID of first tenant using it for debug
        // Exclude constraint if provided
        let used_by_tenant: Option<String> = if let Some(skip_id) = exclude_tenant_id {
            sqlx::query_scalar(
                "SELECT name FROM auth_tenants WHERE active_theme_id = $1 AND id != $2 LIMIT 1"
            )
            .bind(theme_id)
            .bind(skip_id)
            .fetch_optional(&self.db.pool)
            .await?
        } else {
             sqlx::query_scalar(
                "SELECT name FROM auth_tenants WHERE active_theme_id = $1 LIMIT 1"
            )
            .bind(theme_id)
            .fetch_optional(&self.db.pool)
            .await?
        };

        if let Some(tenant_name) = used_by_tenant {
            return Ok(Some(format!("Active in tenant '{}'", tenant_name)));
        }

        // 3. Check System Config (console_theme_*_id) maps to theme NAME
        // IMPORTANT: Only System Themes (tenant_id = None) can be System Defaults.
        // If this is a Tenant Theme, it CANNOT be the System Default (even if name matches).
        if theme_tenant_id.is_some() {
            return Ok(None);
        }
        // We check each key individually to be specific
        // Note: value is JSONB. If it's a string "Pixco Dark", value->>0 or value #>> '{}' gets text.
        // value::text gives "\"Pixco Dark\"".
        // Let's compare against the exact JSON string representation.
        let light_check: Option<String> = sqlx::query_scalar(
            "SELECT value::text FROM sys_configs WHERE key = 'console_theme_light_id' AND value::text = $1"
        )
        // Bind the JSON string representation matches what we store
        .bind(format!("\"{}\"", theme_name))
        .fetch_optional(&self.db.pool)
        .await?;

        if light_check.is_some() {
             return Ok(Some("Active as System Light Theme".to_string()));
        }

        let dark_check: Option<String> = sqlx::query_scalar(
            "SELECT value::text FROM sys_configs WHERE key = 'console_theme_dark_id' AND value::text = $1"
        )
        .bind(format!("\"{}\"", theme_name))
        .fetch_optional(&self.db.pool)
        .await?;
        
        if dark_check.is_some() {
             return Ok(Some("Active as System Dark Theme".to_string()));
        }

        Ok(None)
    }

    async fn get_owner_id(&self) -> AppResult<Option<Uuid>> {
        let owner_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM auth_tenants WHERE parent_id = id LIMIT 1"
        )
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(owner_id)
    }
}
