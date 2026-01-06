// ════════════════════════════════════════════════════════════════════════════
// Plugin Repository - Database Access Layer (Runtime Queries)
// Compatible with: kyx-engine, pixco-customer-app, external-projects
// ════════════════════════════════════════════════════════════════════════════

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::modules::system::domain::plugin::Plugin;

/// Plugin repository for database operations
pub struct PluginRepository {
    pool: PgPool,
}

impl PluginRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Get all plugins for a tenant
    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Plugin>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            FROM sys_plugins
            WHERE tenant_id = $1
            ORDER BY name ASC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        let plugins = rows.into_iter()
            .map(|row| Self::row_to_plugin(&row))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(plugins)
    }
    
    /// Get all active plugins for a tenant
    pub async fn find_active_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Plugin>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            FROM sys_plugins
            WHERE tenant_id = $1 AND is_active = true
            ORDER BY name ASC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        let plugins = rows.into_iter()
            .map(|row| Self::row_to_plugin(&row))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(plugins)
    }

    /// Get all available active plugins for a tenant (including Global and Shared from Parent)
    pub async fn find_available_plugins(&self, tenant_id: Uuid, parent_id: Option<Uuid>) -> Result<Vec<Plugin>> {
        let parent_id = parent_id.unwrap_or(Uuid::nil()); // Use nil if no parent, effectively ignoring that clause
        
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                ui,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            FROM sys_plugins
            WHERE 
                (tenant_id = $1) -- Own plugins
                OR (
                    (visibility = 'global') -- System-wide global plugins
                )
                OR (
                    ($2 != '00000000-0000-0000-0000-000000000000'::uuid) 
                    AND (tenant_id = $2 AND visibility = 'shared') -- Shared from parent
                )
            AND is_active = true
            ORDER BY name ASC
            "#
        )
        .bind(tenant_id)
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;
        
        let plugins = rows.into_iter()
            .map(|row| Self::row_to_plugin(&row))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(plugins)
    }
    
    /// Find plugin by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Plugin>> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            FROM sys_plugins
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(Some(Self::row_to_plugin(&r)?)),
            None => Ok(None),
        }
    }
    
    /// Find plugin by plugin_id (slug) within a tenant
    pub async fn find_by_plugin_id(&self, tenant_id: Uuid, plugin_id: &str) -> Result<Option<Plugin>> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            FROM sys_plugins
            WHERE tenant_id = $1 AND plugin_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(Some(Self::row_to_plugin(&r)?)),
            None => Ok(None),
        }
    }
    
    /// Insert a new plugin
    pub async fn insert(&self, plugin: &Plugin) -> Result<Plugin> {
        let capabilities_json = serde_json::to_value(&plugin.capabilities)?;
        let permissions_json = serde_json::to_value(&plugin.permissions)?;
        let data_access_json = serde_json::to_value(&plugin.data_access)?;
        let network_access_json = serde_json::to_value(&plugin.network_access)?;
        let tags_json = serde_json::to_value(&plugin.tags)?;
        let ui_json = serde_json::to_value(&plugin.ui)?;
        
        let row = sqlx::query(
            r#"
            INSERT INTO sys_plugins (
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                ui,
                installed_at, enabled_at, disabled_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35, $36, $37, 
                $38, $39, $40, $41
            )
            RETURNING 
                id, tenant_id, plugin_id, name, version, description,
                author, author_email, author_website,
                icon, banner, category, tags,
                homepage_url, documentation_url, repository_url, support_url,
                runtime, entry_point, capabilities, permissions, data_access, network_access, config,
                wasm_path, wasm_hash, wasm_size_bytes,
                verified, official, featured, is_core_plugin,
                min_app_version, max_app_version,
                status, is_active, error_message,
                ui,
                installed_at, enabled_at, disabled_at, created_at, updated_at
            "#
        )
        .bind(plugin.id)
        .bind(plugin.tenant_id)
        .bind(&plugin.plugin_id)
        .bind(&plugin.name)
        .bind(&plugin.version)
        .bind(&plugin.description)
        .bind(&plugin.author)
        .bind(&plugin.author_email)
        .bind(&plugin.author_website)
        .bind(&plugin.icon)
        .bind(&plugin.banner)
        .bind(&plugin.category)
        .bind(&tags_json)
        .bind(&plugin.homepage_url)
        .bind(&plugin.documentation_url)
        .bind(&plugin.repository_url)
        .bind(&plugin.support_url)
        .bind(&plugin.runtime)
        .bind(&plugin.entry_point)
        .bind(&capabilities_json)
        .bind(&permissions_json)
        .bind(&data_access_json)
        .bind(&network_access_json)
        .bind(&plugin.config)
        .bind(&plugin.wasm_path)
        .bind(&plugin.wasm_hash)
        .bind(plugin.wasm_size_bytes)
        .bind(plugin.verified)
        .bind(plugin.official)
        .bind(plugin.featured)
        .bind(plugin.is_core_plugin)
        .bind(&plugin.min_app_version)
        .bind(&plugin.max_app_version)
        .bind(&plugin.status)
        .bind(plugin.is_active)
        .bind(&plugin.error_message)
        .bind(&ui_json)
        .bind(plugin.installed_at)
        .bind(plugin.enabled_at)
        .bind(plugin.disabled_at)
        .fetch_one(&self.pool)
        .await?;
        
        Self::row_to_plugin(&row)
    }
    
    /// Enable a plugin
    pub async fn enable(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sys_plugins
            SET is_active = true, status = 'enabled', enabled_at = NOW(), disabled_at = NULL
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Disable a plugin
    pub async fn disable(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sys_plugins
            SET is_active = false, status = 'disabled', disabled_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Update plugin status with error
    pub async fn set_error(&self, id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sys_plugins
            SET status = 'error', is_active = false, error_message = $2
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Update plugin configuration
    pub async fn update_config(&self, id: Uuid, config: JsonValue) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sys_plugins
            SET config = $2
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(config)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Delete a plugin
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query(r#"DELETE FROM sys_plugins WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Log plugin event
    pub async fn log_event(
        &self, 
        plugin_id: Uuid, 
        tenant_id: Uuid, 
        event_type: &str,
        event_data: JsonValue
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sys_plugin_events (plugin_id, tenant_id, event_type, event_data)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(plugin_id)
        .bind(tenant_id)
        .bind(event_type)
        .bind(event_data)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Helper: Convert row to Plugin struct
    fn row_to_plugin(row: &sqlx::postgres::PgRow) -> Result<Plugin> {
        use sqlx::Row;
        
        let tags_json: JsonValue = row.try_get("tags")?;
        let tags: Vec<String> = match tags_json {
            JsonValue::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };
        
        let capabilities_json: JsonValue = row.try_get("capabilities")?;
        let capabilities: Vec<String> = match capabilities_json {
            JsonValue::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };
        
        let permissions_json: JsonValue = row.try_get("permissions")?;
        let permissions: Vec<String> = match permissions_json {
            JsonValue::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };
        
        let data_access_json: JsonValue = row.try_get("data_access")?;
        let data_access: Vec<String> = match data_access_json {
            JsonValue::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };
        
        let network_access_json: JsonValue = row.try_get("network_access")?;
        let network_access: Vec<String> = match network_access_json {
            JsonValue::Array(arr) => arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };

        // Extract UI JSON
        let ui: Option<JsonValue> = row.try_get("ui")?;
        
        Ok(Plugin {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            plugin_id: row.try_get("plugin_id")?,
            name: row.try_get("name")?,
            version: row.try_get("version")?,
            description: row.try_get("description")?,
            author: row.try_get("author")?,
            author_email: row.try_get("author_email")?,
            author_website: row.try_get("author_website")?,
            icon: row.try_get("icon")?,
            banner: row.try_get("banner")?,
            category: row.try_get("category")?,
            tags,
            homepage_url: row.try_get("homepage_url")?,
            documentation_url: row.try_get("documentation_url")?,
            repository_url: row.try_get("repository_url")?,
            support_url: row.try_get("support_url")?,
            runtime: row.try_get("runtime")?,
            entry_point: row.try_get("entry_point")?,
            capabilities,
            permissions,
            data_access,
            network_access,
            config: row.try_get("config")?,
            ui, 
            wasm_path: row.try_get("wasm_path")?,
            wasm_hash: row.try_get("wasm_hash")?,
            wasm_size_bytes: row.try_get("wasm_size_bytes")?,
            verified: row.try_get("verified")?,
            official: row.try_get("official")?,
            featured: row.try_get("featured")?,
            is_core_plugin: row.try_get("is_core_plugin")?,
            min_app_version: row.try_get("min_app_version")?,
            max_app_version: row.try_get("max_app_version")?,
            status: row.try_get("status")?,
            is_active: row.try_get("is_active")?,
            error_message: row.try_get("error_message")?,
            installed_at: row.try_get("installed_at")?,
            enabled_at: row.try_get("enabled_at")?,
            disabled_at: row.try_get("disabled_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
