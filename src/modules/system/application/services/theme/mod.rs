use std::sync::Arc;
use uuid::Uuid;
use crate::core::AppResult;
use crate::modules::system::domain::theme::{Theme, ThemeRepository, ThemeVisibility};
use crate::modules::system::interface::http::dto::theme::{CreateThemeDto, UpdateThemeDto};

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
    pub async fn create_system_theme(&self, name: String, config: serde_json::Value, visibility: ThemeVisibility) -> AppResult<Theme> {
        let dto = CreateThemeDto {
            code: None,
            name,
            description: None,
            config,
            visibility,
            tenant_id: None, // System owned
            author: None,
            preview_url: None,
            logo_url: None,
        };
        self.repo.create(dto).await
    }
    
    // For Tenant attempting to create/fork
    #[allow(dead_code)]
    pub async fn create_tenant_theme(&self, tenant_id: Uuid, name: String, description: Option<String>, config: serde_json::Value) -> AppResult<Theme> {
        let dto = CreateThemeDto {
            code: None,
            name,
            description,
            config,
            visibility: ThemeVisibility::Private, // Always private initially
            tenant_id: Some(tenant_id),
            author: None,
            preview_url: None,
            logo_url: None,
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
    pub async fn update_theme(&self, id: Uuid, dto: UpdateThemeDto) -> AppResult<Theme> {
        // TODO: Enforce permissions (Tenant cannot update System theme)
        self.repo.update(id, dto).await
    }

    #[allow(dead_code)]
    pub async fn delete_theme(&self, id: Uuid) -> AppResult<()> {
        // TODO: Enforce permissions
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
             Err(crate::core::AppError::forbidden("Theme not authorized for this tenant"))
        }
    }
    
    #[allow(dead_code)]
    pub async fn fork_theme(&self, source_theme_id: Uuid, target_tenant_id: Uuid, new_name: String) -> AppResult<Theme> {
        let source = self.repo.find_by_id(source_theme_id).await?;
        if let Some(src) = source {
             // Verify access to source? Assumed yes if they can see it to click Fork.
             // Create new theme
             let dto = CreateThemeDto {
                 code: None,
                 name: new_name,
                 description: Some(format!("Forked from {}", src.name)),
                 config: src.config,
                 visibility: ThemeVisibility::Private,
                 tenant_id: Some(target_tenant_id),
                 author: src.author.clone(),
                 preview_url: None,
                 logo_url: None,
             };
             self.repo.create(dto).await
        } else {
             Err(crate::core::AppError::not_found("Source theme not found"))
        }
    }

    pub async fn is_theme_in_use(&self, theme_id: Uuid, exclude_tenant_id: Option<Uuid>) -> AppResult<Option<String>> {
        self.repo.is_in_use(theme_id, exclude_tenant_id).await
    }
    
    pub async fn seed_default_themes(&self) -> AppResult<()> {
        let available = self.repo.find_available(None).await?;
        
        let folders = vec!["kyx-light", "kyx-dark"];

        for folder in folders {
            let path = format!("assets/themes/presets/{}", folder);
            
            // 1. Read Unified Manifest
            let manifest_path = format!("{}/manifest.json", path);
            let manifest: serde_json::Value = if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                serde_json::from_str(&content).map_err(|e| crate::core::AppError::internal_server_error(format!("Failed to parse {}: {}", manifest_path, e)))?
            } else {
                continue; // Skip if no manifest found
            };

            let name = manifest["name"].as_str().unwrap_or(folder);
            let code = manifest["id"].as_str().map(|s| s.to_string());
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
            
            let existing_theme = available.iter().find(|t| t.name == name && (t.tenant_id == owner_id || t.tenant_id.is_none()));

            // 2. Extract and Bundle CSS
            let mut visited = std::collections::HashSet::new();
            let bundled_css = Box::pin(self.inline_css_recursive(&path, "theme.css", &mut visited)).await?;

            // 3. Prepare Config
            let mut config_val = manifest["config"].clone();
            if config_val.is_null() {
                config_val = serde_json::json!({});
            }

            if let Some(obj) = config_val.as_object_mut() {
                obj.insert("theme_css".to_string(), serde_json::Value::String(bundled_css));
            }

            if let Some(theme) = existing_theme {
                log::info!("🔄 Updating Default {} Theme from unified manifest...", name);
                let update_dto = UpdateThemeDto {
                    code: code.clone(),
                    name: Some(name.to_string()),
                    description: manifest["description"].as_str().map(|s| s.to_string()),
                    config: Some(config_val),
                    visibility: Some(ThemeVisibility::Public),
                    is_active: None,
                    author: author.clone(),
                    preview_url: preview_url.clone(),
                    logo_url: logo_url.clone(),
                };
                self.repo.update(theme.id, update_dto).await?;
            } else {
                log::info!("🌱 Seeding Default {} Theme from unified manifest...", name);
                let dto = CreateThemeDto {
                    code: code.clone(),
                    name: name.to_string(),
                    description: manifest["description"].as_str().map(|s| s.to_string()),
                    config: config_val,
                    visibility: ThemeVisibility::Public,
                    tenant_id: owner_id,
                    author: author,
                    preview_url,
                    logo_url,
                };
                self.repo.create(dto).await?;
            }
        }

        Ok(())
    }

    async fn inline_css_recursive(&self, base_path: &str, file_name: &str, visited: &mut std::collections::HashSet<String>) -> AppResult<String> {
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
                let path_match = if trimmed.contains("url(") {
                    trimmed.split(|c| c == '\'' || c == '"').nth(1)
                } else {
                    trimmed.split(|c| c == '\'' || c == '"').nth(1)
                };

                if let Some(import_path) = path_match {
                    if import_path.starts_with("./") {
                        let sub_file = &import_path[2..];
                        let sub_content = Box::pin(self.inline_css_recursive(base_path, sub_file, visited)).await?;
                        inlined_content.push_str(&sub_content);
                        inlined_content.push('\n');
                        continue;
                    }
                }
            }
            inlined_content.push_str(line);
            inlined_content.push('\n');
        }

        Ok(inlined_content)
    }
}
