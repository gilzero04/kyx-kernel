use crate::core::AppResult;
use crate::core::utils::jwt::Claims;
use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::theme::ThemeService;
use crate::modules::system::interface::http::dto::theme::CreateThemeDto;
use futures_util::StreamExt;
use ntex::web;
use ntex_multipart::Multipart;
use serde::Deserialize;
use std::io::Read;
use uuid::Uuid;

use base64::Engine;

#[derive(Debug, Deserialize)]
pub struct ActivateThemeRequest {
    pub mode: Option<String>, // "light" or "dark"
}

#[derive(Debug, Deserialize)]
pub struct UpdateSharingRequest {
    pub is_shared: bool, // true = broadcast to descendants, false = private
}

// Helper to determine mime type from extension
fn get_mime_type(file_name: &str) -> &'static str {
    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

// Helper to bundle CSS from zip recursively
fn bundle_css_from_zip<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    file_name: &str,
    visited: &mut std::collections::HashSet<String>,
) -> String {
    if visited.contains(file_name) {
        return format!("/* Circular import detected: {} */\n", file_name);
    }
    visited.insert(file_name.to_string());

    let content = {
        let mut file = match archive.by_name(file_name) {
            Ok(f) => f,
            Err(_) => return format!("/* File not found in zip: {} */\n", file_name),
        };

        let mut c = String::new();
        if file.read_to_string(&mut c).is_err() {
            return format!("/* Failed to read file: {} */\n", file_name);
        };
        c
    }; // file is dropped here

    let mut inlined_content = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@import") {
            let path_match = trimmed.split(|c| c == '\'' || c == '"').nth(1);

            if let Some(import_path) = path_match {
                if import_path.starts_with("./") {
                    let sub_file = &import_path[2..];
                    let sub_content = bundle_css_from_zip(archive, sub_file, visited);
                    inlined_content.push_str(&sub_content);
                    inlined_content.push('\n');
                    continue;
                }
            }
        }
        inlined_content.push_str(line);
        inlined_content.push('\n');
    }

    inlined_content
}

// Helper to extract zip
fn extract_theme_config(bytes: &[u8]) -> AppResult<serde_json::Value> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| crate::core::AppError::bad_request(format!("Invalid Zip file: {}", e)))?;

    let content = {
        let mut c = String::new();
        let mut found = false;

        if let Ok(mut file) = archive.by_name("manifest.json") {
            file.read_to_string(&mut c).map_err(|_| {
                crate::core::AppError::internal_server_error("Failed to read manifest.json")
            })?;
            found = true;
        }

        if !found {
            if let Ok(mut file) = archive.by_name("theme.json") {
                file.read_to_string(&mut c).map_err(|_| {
                    crate::core::AppError::internal_server_error("Failed to read theme.json")
                })?;
                found = true;
            }
        }

        if !found {
            return Err(crate::core::AppError::bad_request(
                "Theme manifest (manifest.json or theme.json) not found in zip",
            ));
        }
        c
    }; // any borrows dropped here

    let mut config: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        crate::core::AppError::bad_request(format!("Invalid JSON in manifest: {}", e))
    })?;

    // 2. Handle Preview & Logo conversion to Data URL (Base64)
    // We convert them now so the frontend doesn't need to resolve relative paths
    for field in ["preview", "logo"] {
        if let Some(path) = config.get(field).and_then(|v| v.as_str()) {
            // Only convert if it's a relative path
            if !path.starts_with("http") && !path.starts_with("/") && !path.starts_with("data:") {
                let path_copy = path.to_string();

                // 1. Identify the actual name in the ZIP
                let mut actual_name = None;

                // Try exact
                if archive.by_name(&path_copy).is_ok() {
                    actual_name = Some(path_copy.clone());
                }
                // Try prefixed
                else if archive.by_name(&format!("./{}", path_copy)).is_ok() {
                    actual_name = Some(format!("./{}", path_copy));
                }
                // Try search by filename
                else {
                    let filename = path_copy.split('/').last().unwrap_or(&path_copy);
                    for i in 0..archive.len() {
                        if let Ok(file) = archive.by_index(i) {
                            if file.name().ends_with(filename) {
                                actual_name = Some(file.name().to_string());
                                break;
                            }
                        }
                    }
                }

                // 2. Perform extraction if found
                match actual_name.and_then(|name| archive.by_name(&name).ok()) {
                    Some(mut asset_file) => {
                        let mut buffer = Vec::new();
                        if asset_file.read_to_end(&mut buffer).is_ok() {
                            let mime = get_mime_type(&path_copy);
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
                            let data_url = format!("data:{};base64,{}", mime, b64);
                            config[field] = serde_json::Value::String(data_url);
                            log::debug!(
                                "🖼️ Converted theme asset '{}' to Base64 ({} bytes)",
                                path_copy,
                                buffer.len()
                            );
                        }
                    }
                    None => {
                        log::warn!(
                            "⚠️ Theme asset '{}' defined in manifest but not found in ZIP",
                            path_copy
                        );
                    }
                }
            }
        }
    }

    // 3. Bundle CSS starts from theme.css
    let mut visited = std::collections::HashSet::new();
    let bundled_css = bundle_css_from_zip(&mut archive, "theme.css", &mut visited);

    // Inject bundled CSS into config
    if let Some(obj) = config.get_mut("config").and_then(|c| c.as_object_mut()) {
        obj.insert(
            "theme_css".to_string(),
            serde_json::Value::String(bundled_css),
        );
    } else if let Some(obj) = config.as_object_mut() {
        // Fallback: put it at root of config if no "config" sub-object found
        obj.insert(
            "theme_css".to_string(),
            serde_json::Value::String(bundled_css),
        );
    }

    Ok(config)
}

/// List available themes
#[utoipa::path(
    get,
    path = "/api/v1/public/themes",
    responses(
        (status = 200, description = "List of available themes")
    ),
    tag = "themes"
)]
pub async fn list_themes(
    req: web::HttpRequest,
    service: web::types::State<std::sync::Arc<ThemeService>>,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let claims = req.extensions().get::<Claims>().cloned();
    let tenant_id = claims.map(|c| c.tenant_id);

    // Check if this tenant is the "System Owner"
    // If so, they should see EVERYTHING (pass None) to manage the platform
    // But wait, if they pass None, they see ALL.
    // If they pass Some(OwnerID), they see Public + OwnerID.
    // If there are Private themes from Tenant B, OwnerID won't see them?
    // PLATFORM ADMIN should see everything.
    // My previous logic: "if let Some(tid) = tenant_id" -> checks visibility.
    // "else" (None) -> SELECT * FROM sys_themes. (ALL).

    // So if the user is System Admin, we should pass None?
    // How do we know if they are System Admin?
    // We can check if `tid` == `service.get_owner_id()`.
    // But `ThemeService` might not have `get_owner_id`. `TenantService` does.
    // For now, let's assume if they are querying /api/v1/system/* they are Admin?
    // No, the route is `/api/v1/themes` (from `routers/theme/mod.rs` -> `/themes` resource).

    // Let's rely on standard logic: If you verify as a specific Tenant, you see what that Tenant sees.
    // Usage: "Start by importing a theme... or sync with system library".
    // If I am System Admin, I want to see everything to manage it.

    // For now, let's use the explicit tenant_id.
    // Optimization: If we really want "SuperAdmin Mode" we could add a query param or specific role check.
    // But strictly speaking, respecting tenant barriers is safer.

    let themes = service.list_available_themes(tenant_id).await?;
    let response = ApiResponse::ok(serde_json::json!({ "themes": themes }), "Themes retrieved");
    Ok(web::HttpResponse::Ok().json(&response))
}

/// Import a theme from ZIP file
#[utoipa::path(
    post,
    path = "/api/v1/admin/themes/import",
    responses(
        (status = 201, description = "Theme imported successfully"),
        (status = 400, description = "Invalid theme package")
    ),
    tag = "themes",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn import_theme(
    mut payload: Multipart,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    claims: Claims,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let mut manifest = serde_json::json!({});

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| crate::core::AppError::bad_request("Payload error"))?;

        let cd = field
            .headers()
            .get(&ntex::http::header::CONTENT_DISPOSITION)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        if cd.contains("filename=") {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk
                    .map_err(|_| crate::core::AppError::bad_request("Failed to read chunk"))?;
                bytes.extend_from_slice(&data);
            }

            manifest = extract_theme_config(&bytes)?;
        }
    }

    let name = manifest["name"]
        .as_str()
        .unwrap_or("Imported Theme")
        .to_string();
    let slug = manifest["slug"].as_str().map(|s| s.to_string());
    let description = manifest["description"].as_str().map(|s| s.to_string());
    let author = manifest["author"].as_str().map(|s| s.to_string());
    let preview_url = manifest["preview"].as_str().map(|s| s.to_string());
    let logo_url = manifest["logo"].as_str().map(|s| s.to_string());

    // Extract the internal "config" block which now includes bundled theme_css
    let config = if manifest["config"].is_object() {
        manifest["config"].clone()
    } else {
        // Fallback for flat manifests
        let mut flat_config = manifest.clone();
        if let Some(obj) = flat_config.as_object_mut() {
            obj.remove("name");
            obj.remove("description");
            obj.remove("author");
            obj.remove("preview");
            obj.remove("logo");
        }
        flat_config
    };

    let tenant_id = Some(claims.tenant_id);

    // Create the theme
    let dto = CreateThemeDto {
        id: None, // Imported themes get auto-generated ID
        slug,
        name,
        description,
        config,
        is_shared: false,
        tenant_id,
        author,
        preview_url,
        logo_url,
        version: manifest["version"].as_str().map(|s| s.to_string()),
    };

    let theme = service.create_theme(dto).await?;
    let response = ApiResponse::created(&theme, "Theme imported");
    Ok(web::HttpResponse::Created().json(&response))
}

/// Activate a theme for a specific mode
#[utoipa::path(
    patch,
    path = "/api/v1/admin/themes/{id}/activate",
    request_body = ActivateThemeRequest,
    responses(
        (status = 200, description = "Theme activated"),
        (status = 403, description = "Permission denied"),
        (status = 404, description = "Theme not found")
    ),
    tag = "themes",
    params(
        ("id" = Uuid, Path, description = "Theme ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_active_theme(
    path: web::types::Path<Uuid>, // Theme ID
    body: web::types::Json<ActivateThemeRequest>,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    db: web::types::State<std::sync::Arc<crate::core::infrastructure::database::Database>>,
    claims: Claims, // Injected by middleware - used for tenant_id
) -> Result<web::HttpResponse, crate::core::AppError> {
    // Note: Permission check (theme:activate) is handled by RequirePermission middleware
    // See: src/modules/system/interface/http/routers/theme/mod.rs

    let theme_id = path.into_inner();
    let mode = body.mode.clone().unwrap_or_else(|| "light".to_string());

    // Verify theme exists
    let theme = service.get_theme(theme_id).await?;
    if theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }

    let theme = theme.unwrap();

    // Update the caller's tenant branding in sys_brandings
    // Use claims.tenant_id to get the correct branding for this tenant
    let branding_id: Option<sqlx::types::Uuid> = sqlx::query_scalar(
        r#"
        SELECT branding_id FROM auth_tenants 
        WHERE id = $1 AND deleted_at IS NULL
        LIMIT 1
    "#,
    )
    .bind(claims.tenant_id)
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| crate::core::AppError::internal_server_error(format!("DB error: {}", e)))?
    .flatten();

    let branding_id = match branding_id {
        Some(id) => id,
        None => {
            return Err(crate::core::AppError::not_found(
                "Tenant branding not found",
            ));
        }
    };

    // Update console theme AND set workspace/app defaults if they are NULL
    // This ensures that when SuperAdmin sets console theme, workspace/app have defaults too
    // DB columns: theme_workspace_light_id, theme_app_light_id (NOT workspace_theme_*)
    let sql = if mode == "dark" {
        r#"
        UPDATE sys_brandings SET 
            theme_dark_id = $1,
            theme_workspace_dark_id = COALESCE(theme_workspace_dark_id, $1),
            theme_app_dark_id = COALESCE(theme_app_dark_id, $1),
            updated_at = NOW() 
        WHERE id = $2
        "#
    } else {
        r#"
        UPDATE sys_brandings SET 
            theme_light_id = $1,
            theme_workspace_light_id = COALESCE(theme_workspace_light_id, $1),
            theme_app_light_id = COALESCE(theme_app_light_id, $1),
            updated_at = NOW() 
        WHERE id = $2
        "#
    };

    sqlx::query(sql)
        .bind(theme_id) // Theme UUID
        .bind(branding_id)
        .execute(&db.pool)
        .await
        .map_err(|e| {
            crate::core::AppError::internal_server_error(format!(
                "Failed to update branding: {}",
                e
            ))
        })?;

    log::info!(
        "🎨 Theme '{}' activated for {} mode by tenant {} (branding_id: {}) - also set defaults for workspace/app if empty",
        theme.name,
        mode,
        claims.tenant_id,
        branding_id
    );

    let response = ApiResponse::ok(
        serde_json::json!({
            "theme_id": theme_id,
            "theme_name": theme.name,
            "mode": mode
        }),
        "Theme activated",
    );
    Ok(web::HttpResponse::Ok().json(&response))
}

/// Delete a theme
#[utoipa::path(
    delete,
    path = "/api/v1/admin/themes/{id}",
    responses(
        (status = 204, description = "Theme deleted"),
        (status = 400, description = "Cannot delete theme in use"),
        (status = 404, description = "Theme not found")
    ),
    tag = "themes",
    params(
        ("id" = Uuid, Path, description = "Theme ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_theme(
    path: web::types::Path<Uuid>,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    claims: Claims,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let theme_id = path.into_inner();

    let actor_tenant_id = Some(claims.tenant_id);

    // Verify theme exists first
    let theme = service.get_theme(theme_id).await?;
    if theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }

    let theme = theme.unwrap();

    // Check Ownership (unless System Admin? But we treat System Admin as just another tenant usually)
    // If actor is None (unknown), deny?
    // If actor is Some(ID), must match theme.tenant_id.
    // Exception: System Admin (if we identify them mechanism).

    // Simplify: If theme has tenant_id, actor MUST match it.
    // If theme has NO tenant_id (System Default), NO ONE can delete it (except maybe manual DB).

    if theme.tenant_id.is_none() {
        return Err(crate::core::AppError::forbidden(
            "Cannot delete system default themes.",
        ));
    }

    if let Some(actor_id) = actor_tenant_id {
        if Some(actor_id) != theme.tenant_id {
            return Err(crate::core::AppError::forbidden(
                "You do not have permission to delete this theme (Not Owner).",
            ));
        }
    } else {
        return Err(crate::core::AppError::unauthorized(
            "Authentication required",
        ));
    }

    // Also check usage before delete!
    // For delete, we generally want NO usage at all, even by owner?
    // "Theme ที่ add เข้ามาถ้าไม่มีคนใช้... สามารถลบทิ้งได้"
    // If owner is using it, they probably need to switch theme before deleting it.
    // So pass None (don't exclude anyone).
    if let Some(reason) = service.is_theme_in_use(theme_id, None).await? {
        return Err(crate::core::AppError::bad_request(format!(
            "Cannot delete theme. Reason: {}",
            reason
        )));
    }

    // Rule 5: Shared themes cannot be deleted. Must be Un-shared first.
    if theme.is_shared {
        return Err(crate::core::AppError::bad_request(
            "Cannot delete a shared theme. Please Un-share it first.",
        ));
    }

    service.delete_theme(theme_id).await?;

    log::info!("🗑️ Theme '{}' deleted", theme.name);

    let response = ApiResponse::ok(
        serde_json::json!({
            "deleted": true,
            "theme_id": theme_id
        }),
        "Theme deleted",
    );
    Ok(web::HttpResponse::Ok().json(&response))
}

/// Update theme sharing status
#[utoipa::path(
    patch,
    path = "/api/v1/admin/themes/{id}/sharing",
    request_body = UpdateSharingRequest,
    responses(
        (status = 200, description = "Sharing status updated"),
        (status = 400, description = "Cannot change sharing"),
        (status = 404, description = "Theme not found")
    ),
    tag = "themes",
    params(
        ("id" = Uuid, Path, description = "Theme ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_sharing(
    path: web::types::Path<Uuid>,
    body: web::types::Json<UpdateSharingRequest>,
    service: web::types::State<std::sync::Arc<ThemeService>>,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let theme_id = path.into_inner();
    let is_shared = body.is_shared;

    let dto = crate::modules::system::interface::http::dto::theme::UpdateThemeDto {
        slug: None,
        is_shared: Some(is_shared),
        name: None,
        description: None,
        config: None,
        is_active: None,
        author: None,
        preview_url: None,
        logo_url: None,
        version: None,
    };

    // Verify theme exists first
    let theme = service.get_theme(theme_id).await?;
    if theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }

    let theme = theme.unwrap();

    // Constraints for Un-sharing (is_shared: true -> false)
    if !is_shared {
        // 1. Cannot un-share System Default Themes (True System Themes have no tenant_id)
        if theme.tenant_id.is_none() {
            return Err(crate::core::AppError::forbidden(
                "Cannot un-share system default themes.",
            ));
        }

        // 2. Cannot un-share if currently in use (EXCEPT by the owner themselves)
        if let Some(reason) = service.is_theme_in_use(theme_id, theme.tenant_id).await? {
            return Err(crate::core::AppError::bad_request(format!(
                "Cannot un-share theme. Reason: {}",
                reason
            )));
        }
    }

    service.update_theme(theme_id, dto).await?;

    let response = ApiResponse::ok(
        serde_json::json!({
            "theme_id": theme_id,
            "is_shared": is_shared
        }),
        "Sharing status updated",
    );
    Ok(web::HttpResponse::Ok().json(&response))
}

/// Update a system theme by uploading a new ZIP file
/// Only SuperAdmin + Owner can update system themes
#[utoipa::path(
    post,
    path = "/api/v1/admin/themes/{id}/update",
    responses(
        (status = 200, description = "Theme updated successfully"),
        (status = 400, description = "Invalid theme package"),
        (status = 403, description = "Permission denied - SuperAdmin Owner only"),
        (status = 404, description = "Theme not found")
    ),
    tag = "themes",
    params(
        ("id" = Uuid, Path, description = "Theme ID to update")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_theme(
    path: web::types::Path<Uuid>,
    mut payload: Multipart,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    db: web::types::State<std::sync::Arc<crate::core::infrastructure::database::Database>>,
    claims: Claims,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let theme_id = path.into_inner();

    // 1. Verify caller is SuperAdmin + Owner
    let role = claims.role.to_lowercase();
    if role != "superadmin" {
        return Err(crate::core::AppError::forbidden(
            "Only SuperAdmin can update system themes",
        ));
    }

    // Check if caller's tenant is the system owner (root tenant has parent_id = id)
    let is_owner: Option<bool> = sqlx::query_scalar(
        r#"
        SELECT (parent_id = id) AS is_owner FROM auth_tenants 
        WHERE id = $1 AND deleted_at IS NULL
        LIMIT 1
    "#,
    )
    .bind(claims.tenant_id)
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| crate::core::AppError::internal_server_error(format!("DB error: {}", e)))?
    .flatten();

    if !is_owner.unwrap_or(false) {
        return Err(crate::core::AppError::forbidden(
            "Only the system owner can update system themes",
        ));
    }

    // 2. Verify theme exists
    let existing_theme = service.get_theme(theme_id).await?;
    if existing_theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }
    let existing_theme = existing_theme.unwrap();

    // 3. Extract and validate the uploaded ZIP
    let mut manifest = serde_json::json!({});

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| crate::core::AppError::bad_request("Payload error"))?;

        let cd = field
            .headers()
            .get(&ntex::http::header::CONTENT_DISPOSITION)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        if cd.contains("filename=") {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk
                    .map_err(|_| crate::core::AppError::bad_request("Failed to read chunk"))?;
                bytes.extend_from_slice(&data);
            }

            manifest = extract_theme_config(&bytes)?;
        }
    }

    // 4. Prepare update DTO with new config (bundled CSS)
    // For system themes (is_system=true), preserve existing name/description/author
    // Only update config (CSS styles) for system themes
    let is_system = existing_theme.tenant_id.is_none() || existing_theme.is_system;

    let name = if is_system {
        None // Keep existing name for system themes
    } else {
        manifest["name"].as_str().map(|s| s.to_string())
    };

    let description = if is_system {
        None // Keep existing description for system themes
    } else {
        manifest["description"].as_str().map(|s| s.to_string())
    };

    let author = if is_system {
        None // Keep existing author for system themes
    } else {
        manifest["author"].as_str().map(|s| s.to_string())
    };

    let preview_url = manifest["preview"].as_str().map(|s| s.to_string());
    let logo_url = manifest["logo"].as_str().map(|s| s.to_string());

    // Extract the internal "config" block which now includes bundled theme_css
    let config = if manifest["config"].is_object() {
        Some(manifest["config"].clone())
    } else {
        // Fallback for flat manifests
        let mut flat_config = manifest.clone();
        if let Some(obj) = flat_config.as_object_mut() {
            obj.remove("name");
            obj.remove("description");
            obj.remove("author");
            obj.remove("preview");
            obj.remove("logo");
        }
        Some(flat_config)
    };

    let dto = crate::modules::system::interface::http::dto::theme::UpdateThemeDto {
        slug: None,      // Keep existing slug
        is_shared: None, // Keep existing sharing status
        name,
        description,
        config,
        is_active: None,
        author,
        preview_url,
        logo_url,
        version: manifest["version"].as_str().map(|s| s.to_string()),
    };

    // 5. Update the theme
    service.update_theme(theme_id, dto).await?;

    log::info!(
        "🔄 Theme '{}' (ID: {}) updated by SuperAdmin Owner (tenant: {})",
        existing_theme.name,
        theme_id,
        claims.tenant_id
    );

    let response = ApiResponse::ok(
        serde_json::json!({
            "theme_id": theme_id,
            "theme_name": existing_theme.name,
            "message": "Theme updated successfully. Refresh to see changes."
        }),
        "Theme updated",
    );
    Ok(web::HttpResponse::Ok().json(&response))
}
