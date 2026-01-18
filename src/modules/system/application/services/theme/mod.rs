use crate::core::AppResult;
use crate::modules::system::domain::theme::{Theme, ThemeRepository};
use crate::modules::system::interface::http::dto::theme::{CreateThemeDto, UpdateThemeDto};
use std::sync::Arc;
use uuid::Uuid;

pub struct ThemeService {
    repo: Arc<dyn ThemeRepository>,
}

impl ThemeService {
    pub fn new(repo: Arc<dyn ThemeRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_theme(&self, dto: CreateThemeDto) -> AppResult<Theme> {
        // Validation could go here (e.g. check if restricted is valid)
        self.repo.create(dto).await
    }

    // For System Admin
    #[allow(dead_code)]
    pub async fn create_system_theme(
        &self,
        name: String,
        config: serde_json::Value,
        is_shared: bool,
    ) -> AppResult<Theme> {
        let dto = CreateThemeDto {
            id: None, // Auto-generate
            slug: None,
            name,
            description: None,
            config,
            is_shared,
            tenant_id: None, // System owned
            author: None,
            preview_url: None,
            logo_url: None,
            version: None,
        };
        self.repo.create(dto).await
    }

    // For Tenant attempting to create/fork
    #[allow(dead_code)]
    pub async fn create_tenant_theme(
        &self,
        tenant_id: Uuid,
        name: String,
        description: Option<String>,
        config: serde_json::Value,
    ) -> AppResult<Theme> {
        let dto = CreateThemeDto {
            id: None, // Auto-generate
            slug: None,
            name,
            description,
            config,
            is_shared: false, // Always private (not shared) initially
            tenant_id: Some(tenant_id),
            author: None,
            preview_url: None,
            logo_url: None,
            version: None,
        };
        self.repo.create(dto).await
    }

    pub async fn list_available_themes(&self, tenant_id: Option<Uuid>) -> AppResult<Vec<Theme>> {
        self.repo.find_available(tenant_id).await
    }

    #[allow(dead_code)]
    pub async fn get_theme(&self, id: Uuid) -> AppResult<Option<Theme>> {
        self.repo.find_by_id(id).await
    }

    #[allow(dead_code)]
    pub async fn update_theme(
        &self,
        id: Uuid,
        dto: UpdateThemeDto,
        requester_tenant_id: Option<Uuid>,
    ) -> AppResult<Theme> {
        // Permission check: Validate requester can update this theme
        // - System themes (tenant_id = None) can only be updated by platform admin
        // - Tenant themes can only be updated by the owning tenant
        if let Some(existing) = self.repo.find_by_id(id).await? {
            match (existing.tenant_id, requester_tenant_id) {
                // System theme: Only platform admin (None) can update
                (None, None) => {} // Platform admin - allowed
                (None, Some(_)) => {
                    return Err(crate::core::AppError::forbidden(
                        "Cannot update system theme - only platform admin allowed",
                    ));
                }
                // Tenant theme: Only owning tenant can update
                (Some(owner), Some(requester)) if owner == requester => {} // Same tenant - allowed
                (Some(_), _) => {
                    return Err(crate::core::AppError::forbidden(
                        "Cannot update theme owned by another tenant",
                    ));
                }
            }
        }
        self.repo.update(id, dto).await
    }

    #[allow(dead_code)]
    pub async fn delete_theme(&self, id: Uuid, requester_tenant_id: Option<Uuid>) -> AppResult<()> {
        // Permission check: Validate requester can delete this theme
        // - System themes cannot be deleted
        // - Tenant themes can only be deleted by the owning tenant
        if let Some(existing) = self.repo.find_by_id(id).await? {
            if existing.is_system {
                return Err(crate::core::AppError::forbidden(
                    "Cannot delete system theme",
                ));
            }
            match (existing.tenant_id, requester_tenant_id) {
                (None, _) => {
                    return Err(crate::core::AppError::forbidden(
                        "Cannot delete platform-level theme",
                    ));
                }
                (Some(owner), Some(requester)) if owner == requester => {} // Same tenant - allowed
                (Some(_), _) => {
                    return Err(crate::core::AppError::forbidden(
                        "Cannot delete theme owned by another tenant",
                    ));
                }
            }
        }
        self.repo.delete(id).await
    }

    #[allow(dead_code)]
    pub async fn set_active_theme(&self, tenant_id: Uuid, theme_id: Uuid) -> AppResult<()> {
        // Verify eligibility first
        let available = self.repo.find_available(Some(tenant_id)).await?;
        if available.iter().any(|t| t.id == theme_id) {
            self.repo.set_active_theme(tenant_id, theme_id).await
        } else {
            // Theme not accessible
            Err(crate::core::AppError::forbidden(
                "Theme not authorized for this tenant",
            ))
        }
    }

    #[allow(dead_code)]
    pub async fn fork_theme(
        &self,
        source_theme_id: Uuid,
        target_tenant_id: Uuid,
        new_name: String,
    ) -> AppResult<Theme> {
        let source = self.repo.find_by_id(source_theme_id).await?;
        if let Some(src) = source {
            // Verify access to source? Assumed yes if they can see it to click Fork.
            // Create new theme
            let dto = CreateThemeDto {
                id: None, // Forked themes get auto-generated ID
                slug: None,
                name: new_name,
                description: Some(format!("Forked from {}", src.name)),
                config: src.config,
                is_shared: false,
                tenant_id: Some(target_tenant_id),
                author: src.author.clone(),
                preview_url: None,
                logo_url: None,
                version: None,
            };
            self.repo.create(dto).await
        } else {
            Err(crate::core::AppError::not_found("Source theme not found"))
        }
    }

    pub async fn is_theme_in_use(
        &self,
        theme_id: Uuid,
        exclude_tenant_id: Option<Uuid>,
    ) -> AppResult<Option<String>> {
        self.repo.is_in_use(theme_id, exclude_tenant_id).await
    }

    pub async fn seed_default_themes(&self) -> AppResult<()> {
        let available = self.repo.find_available(None).await?;

        // Debug: Log current working directory
        let cwd = std::env::current_dir().unwrap_or_default();
        log::info!("🌱 [ThemeSeed] Starting theme seeding from CWD: {:?}", cwd);

        let folders = vec!["kyx-light", "kyx-dark"];

        for folder in folders {
            let path = format!("assets/themes/presets/{}", folder);

            // 1. Read Unified Manifest
            let manifest_path = format!("{}/manifest.json", path);
            let manifest: serde_json::Value =
                if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                    serde_json::from_str(&content).map_err(|e| {
                        crate::core::AppError::internal_server_error(format!(
                            "Failed to parse {}: {}",
                            manifest_path, e
                        ))
                    })?
                } else {
                    continue; // Skip if no manifest found
                };

            let name = manifest["name"].as_str().unwrap_or(folder);
            // manifest.id = UUID string (plain UUID, not kyx-theme- prefix)
            // manifest.slug = "kyx-dark" → human-readable identifier
            let manifest_id = manifest["id"].as_str();
            let theme_uuid = manifest_id.and_then(|id| Uuid::parse_str(id).ok());
            let slug = manifest["slug"].as_str().map(|s| s.to_string());
            let author = manifest["author"].as_str().map(|s| s.to_string());

            // Check for preview image
            let preview_filename = manifest["preview"].as_str().unwrap_or("images/preview.png");
            let preview_path = format!("{}/{}", path, preview_filename);
            let preview_url = if tokio::fs::try_exists(&preview_path).await.unwrap_or(false) {
                Some(format!("/themes/presets/{}/{}", folder, preview_filename))
            } else {
                None
            };

            // Check for logo image
            let logo_filename = manifest["logo"].as_str().unwrap_or("images/logo.png");
            let logo_path = format!("{}/{}", path, logo_filename);
            let logo_url = if tokio::fs::try_exists(&logo_path).await.unwrap_or(false) {
                Some(format!("/themes/presets/{}/{}", folder, logo_filename))
            } else {
                None
            };

            let owner_id = self.repo.get_owner_id().await.unwrap_or(None);

            let existing_theme = available
                .iter()
                .find(|t| t.name == name && (t.tenant_id == owner_id || t.tenant_id.is_none()));

            // Extract version from manifest (for comparison)
            let manifest_version = manifest["version"].as_str().map(|s| s.to_string());

            // 2. Extract and Bundle CSS
            let mut visited = std::collections::HashSet::new();
            let bundled_css =
                Box::pin(self.inline_css_recursive(&path, "theme.css", &mut visited)).await?;

            // Debug: Log CSS bundling result
            log::info!(
                "📦 [ThemeSeed] {} - Bundled CSS length: {} chars",
                name,
                bundled_css.len()
            );
            if bundled_css.len() < 100 {
                log::warn!(
                    "⚠️ [ThemeSeed] {} - CSS seems too short! Content: {}",
                    name,
                    &bundled_css[..bundled_css.len().min(200)]
                );
            }

            // 3. Prepare Config
            let mut config_val = manifest["config"].clone();
            if config_val.is_null() {
                config_val = serde_json::json!({});
            }

            if let Some(obj) = config_val.as_object_mut() {
                obj.insert(
                    "theme_css".to_string(),
                    serde_json::Value::String(bundled_css),
                );
            }

            if let Some(theme) = existing_theme {
                // VERSION CHECK: Only update if manifest version > DB version
                // If DB has a higher or equal version, it means user uploaded via API
                let should_update = match (&manifest_version, &theme.version) {
                    (Some(manifest_v), Some(db_v)) => {
                        // Compare semantic versions
                        self.compare_versions(manifest_v, db_v) > 0
                    }
                    (Some(_), None) => true, // DB has no version, update
                    (None, _) => false,      // Manifest has no version, skip update
                };

                // SELF-HEALING: Force update if theme_css is missing or empty
                // This fixes themes created by migration without bundled CSS
                let existing_css = theme
                    .config
                    .get("theme_css")
                    .and_then(|v| v.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0);
                let needs_css_fix = existing_css < 1000; // CSS should be > 1000 chars

                if should_update || needs_css_fix {
                    if needs_css_fix && !should_update {
                        log::info!(
                            "🔧 Fixing {} Theme - CSS was missing/empty ({} chars)",
                            name,
                            existing_css
                        );
                    } else {
                        log::info!(
                            "🔄 Updating Default {} Theme (v{} -> v{})...",
                            name,
                            theme.version.as_deref().unwrap_or("?"),
                            manifest_version.as_deref().unwrap_or("?")
                        );
                    }
                    let update_dto = UpdateThemeDto {
                        slug: slug.clone(),
                        name: Some(name.to_string()),
                        description: manifest["description"].as_str().map(|s| s.to_string()),
                        config: Some(config_val),
                        is_shared: Some(true),
                        is_active: None,
                        author: author.clone(),
                        preview_url: preview_url.clone(),
                        logo_url: logo_url.clone(),
                        version: manifest_version.clone(),
                    };
                    self.repo.update(theme.id, update_dto).await?;
                } else {
                    log::info!(
                        "⏭️ Skipping {} Theme - DB version ({}) >= manifest version ({})",
                        name,
                        theme.version.as_deref().unwrap_or("custom"),
                        manifest_version.as_deref().unwrap_or("none")
                    );
                }
            } else {
                log::info!(
                    "🌱 Seeding Default {} Theme v{} from unified manifest...",
                    name,
                    manifest_version.as_deref().unwrap_or("?")
                );
                let dto = CreateThemeDto {
                    id: theme_uuid, // Use ID from manifest.json (plain UUID)
                    slug: slug.clone(),
                    name: name.to_string(),
                    description: manifest["description"].as_str().map(|s| s.to_string()),
                    config: config_val,
                    is_shared: true,
                    tenant_id: owner_id,
                    author,
                    preview_url,
                    logo_url,
                    version: manifest_version,
                };
                self.repo.create(dto).await?;
            }
        }

        Ok(())
    }

    async fn inline_css_recursive(
        &self,
        base_path: &str,
        file_name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> AppResult<String> {
        let full_path = format!("{}/{}", base_path, file_name);
        if visited.contains(&full_path) {
            return Ok(format!("/* Circular import detected: {} */\n", file_name));
        }
        visited.insert(full_path.clone());

        let content = match tokio::fs::read_to_string(&full_path).await {
            Ok(c) => c,
            Err(_) => return Ok(format!("/* File not found: {} */\n", file_name)),
        };

        let mut inlined_content = String::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@import") {
                // Handle different import styles: @import './file.css'; @import url('./file.css');
                // Both styles use ' or " for quoting the path
                let path_match = trimmed.split(['\'', '"']).nth(1);

                if let Some(import_path) = path_match
                    && let Some(sub_file) = import_path.strip_prefix("./") {
                        let sub_content =
                            Box::pin(self.inline_css_recursive(base_path, sub_file, visited))
                                .await?;
                        inlined_content.push_str(&sub_content);
                        inlined_content.push('\n');
                        continue;
                    }
            }
            inlined_content.push_str(line);
            inlined_content.push('\n');
        }

        Ok(inlined_content)
    }

    /// Compare semantic versions (e.g., "1.0.0" vs "1.1.0")
    /// Returns: 1 if a > b, -1 if a < b, 0 if equal
    fn compare_versions(&self, a: &str, b: &str) -> i32 {
        let parse_parts =
            |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

        let parts_a = parse_parts(a);
        let parts_b = parse_parts(b);

        let max_len = parts_a.len().max(parts_b.len());

        for i in 0..max_len {
            let pa = parts_a.get(i).copied().unwrap_or(0);
            let pb = parts_b.get(i).copied().unwrap_or(0);

            if pa > pb {
                return 1;
            } else if pa < pb {
                return -1;
            }
        }

        0 // Equal
    }
}
