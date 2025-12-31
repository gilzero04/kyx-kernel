use ntex::web;
use ntex_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Read;
use serde::Deserialize;
use uuid::Uuid;
use crate::core::AppResult;
use crate::core::infrastructure::config_service::ConfigService;
use crate::modules::system::application::services::theme::ThemeService;
use crate::modules::system::domain::theme::ThemeVisibility;
use crate::modules::system::interface::http::dto::theme::CreateThemeDto;
use crate::core::utils::jwt::Claims;

use base64::Engine;

#[derive(Debug, Deserialize)]
pub struct ActivateThemeRequest {
    pub mode: Option<String>, // "light" or "dark"
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisibilityRequest {
    pub visibility: String, // "public", "private", "shared"
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
    visited: &mut std::collections::HashSet<String>
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
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| 
        crate::core::AppError::bad_request(format!("Invalid Zip file: {}", e))
    )?;

    let content = {
        let mut c = String::new();
        let mut found = false;
        
        if let Ok(mut file) = archive.by_name("manifest.json") {
            file.read_to_string(&mut c).map_err(|_| 
                crate::core::AppError::internal_server_error("Failed to read manifest.json")
            )?;
            found = true;
        }
        
        if !found {
            if let Ok(mut file) = archive.by_name("theme.json") {
                file.read_to_string(&mut c).map_err(|_| 
                    crate::core::AppError::internal_server_error("Failed to read theme.json")
                )?;
                found = true;
            }
        }
        
        if !found {
            return Err(crate::core::AppError::bad_request("Theme manifest (manifest.json or theme.json) not found in zip"));
        }
        c
    }; // any borrows dropped here

    let mut config: serde_json::Value = serde_json::from_str(&content).map_err(|e| 
        crate::core::AppError::bad_request(format!("Invalid JSON in manifest: {}", e))
    )?;

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
                            log::debug!("🖼️ Converted theme asset '{}' to Base64 ({} bytes)", path_copy, buffer.len());
                        }
                    },
                    None => {
                        log::warn!("⚠️ Theme asset '{}' defined in manifest but not found in ZIP", path_copy);
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
        obj.insert("theme_css".to_string(), serde_json::Value::String(bundled_css));
    } else if let Some(obj) = config.as_object_mut() {
        // Fallback: put it at root of config if no "config" sub-object found
        obj.insert("theme_css".to_string(), serde_json::Value::String(bundled_css));
    }

    Ok(config)
}

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
    Ok(web::HttpResponse::Ok().json(&themes))
}

pub async fn import_theme(
    mut payload: Multipart,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    claims: Claims,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let mut manifest = serde_json::json!({});
    
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| crate::core::AppError::bad_request("Payload error"))?;
        
        let cd = field.headers().get(&ntex::http::header::CONTENT_DISPOSITION)
             .and_then(|h| h.to_str().ok())
             .unwrap_or("");
             
        if cd.contains("filename=") {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|_| crate::core::AppError::bad_request("Failed to read chunk"))?;
                bytes.extend_from_slice(&data);
            }

            manifest = extract_theme_config(&bytes)?;
        }
    }

    let name = manifest["name"].as_str().unwrap_or("Imported Theme").to_string();
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
        name,
        description,
        config,
        visibility: ThemeVisibility::Private,
        tenant_id,
        author,
        preview_url,
        logo_url,
    };

    let theme = service.create_theme(dto).await?;
    Ok(web::HttpResponse::Created().json(&theme))
}

pub async fn set_active_theme(
    path: web::types::Path<Uuid>, // Theme ID
    body: web::types::Json<ActivateThemeRequest>,
    service: web::types::State<std::sync::Arc<ThemeService>>,
    config_service: web::types::State<std::sync::Arc<ConfigService>>,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let theme_id = path.into_inner();
    let mode = body.mode.clone().unwrap_or_else(|| "light".to_string());
    
    // Verify theme exists
    let theme = service.get_theme(theme_id).await?;
    if theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }
    
    let theme = theme.unwrap();
    
    // Update the appropriate config key based on mode
    let config_key = if mode == "dark" {
        "theme_dark_id"
    } else {
        "theme_light_id"
    };
    
    // Use theme ID as the identifier (more robust than name)
    config_service.set(config_key, serde_json::Value::String(theme.id.to_string())).await.map_err(|e| 
        crate::core::AppError::internal_server_error(format!("Failed to update config: {}", e))
    )?;
    
    log::info!("🎨 Theme '{}' activated for {} mode", theme.name, mode);
    
    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "updated",
        "theme_id": theme_id,
        "theme_name": theme.name,
        "mode": mode
    })))
}

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
         return Err(crate::core::AppError::forbidden("Cannot delete system default themes."));
    }
    
    if let Some(actor_id) = actor_tenant_id {
        if Some(actor_id) != theme.tenant_id {
             return Err(crate::core::AppError::forbidden("You do not have permission to delete this theme (Not Owner)."));
        }
    } else {
         return Err(crate::core::AppError::unauthorized("Authentication required"));
    }
    
    // Also check usage before delete!
    // For delete, we generally want NO usage at all, even by owner?
    // "Theme ที่ add เข้ามาถ้าไม่มีคนใช้... สามารถลบทิ้งได้"
    // If owner is using it, they probably need to switch theme before deleting it.
    // So pass None (don't exclude anyone).
    if let Some(reason) = service.is_theme_in_use(theme_id, None).await? {
         return Err(crate::core::AppError::bad_request(format!("Cannot delete theme. Reason: {}", reason)));
    }
    
    // Rule 5: Shared themes cannot be deleted. Must be Un-shared (made Private) first.
    if matches!(theme.visibility, crate::modules::system::domain::theme::entity::ThemeVisibility::Public) {
        return Err(crate::core::AppError::bad_request("Cannot delete a shared (Public) theme. Please Un-share it first."));
    }
    
    service.delete_theme(theme_id).await?;
    
    log::info!("🗑️ Theme '{}' deleted", theme.name);
    
    Ok(web::HttpResponse::NoContent().finish())
}

pub async fn set_visibility(
    path: web::types::Path<Uuid>,
    body: web::types::Json<UpdateVisibilityRequest>,
    service: web::types::State<std::sync::Arc<ThemeService>>,
) -> Result<web::HttpResponse, crate::core::AppError> {
    let theme_id = path.into_inner();
    let visibility = match body.visibility.to_lowercase().as_str() {
        "public" => ThemeVisibility::Public,
        "private" => ThemeVisibility::Private,
         _ => return Err(crate::core::AppError::bad_request("Invalid visibility mode. Must be 'public' or 'private'.")),
    };

    let dto = crate::modules::system::interface::http::dto::theme::UpdateThemeDto {
        visibility: Some(visibility),
        name: None,
        description: None,
        config: None,
        is_active: None,
        author: None,
        preview_url: None,
        logo_url: None,
    };
    
    // Verify theme exists first
    let theme = service.get_theme(theme_id).await?;
    if theme.is_none() {
        return Err(crate::core::AppError::not_found("Theme not found"));
    }
    
    let theme = theme.unwrap();

    // Constraints for Un-sharing (Public -> Private)
    if matches!(visibility, ThemeVisibility::Private) {
        // 1. Cannot un-share System Default Themes (True System Themes have no tenant_id)
        if theme.tenant_id.is_none() {
             return Err(crate::core::AppError::forbidden("Cannot un-share system default themes."));
        }
        
        // 2. Cannot un-share if currently in use (EXCEPT by the owner themselves)
        // We pass `theme.tenant_id` as the excluded tenant. If `active_theme_id` matches this tenant, it's ignored.
        // If ANY ONE ELSE is using it, `is_theme_in_use` returns Some(reason).
        if let Some(reason) = service.is_theme_in_use(theme_id, theme.tenant_id).await? {
             return Err(crate::core::AppError::bad_request(format!("Cannot un-share theme. Reason: {}", reason)));
        }
    }

    service.update_theme(theme_id, dto).await?;
    
    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "updated",
        "theme_id": theme_id,
        "visibility": body.visibility
    })))
}
