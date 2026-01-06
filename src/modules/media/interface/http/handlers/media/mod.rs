use ntex::web;
use ntex_multipart::Multipart;
use futures_util::StreamExt;
use uuid::Uuid;
use crate::core::infrastructure::audit::AuditService;
use crate::core::utils::jwt::Claims;
use crate::modules::media::application::services::media::MediaService;
use crate::modules::media::interface::http::dto::media::{CreateFolderRequest, AssetQuery};
use std::sync::Arc;
use std::path::Path;

// DELETED: Manual extraction replaced by direct Claims injection

/// Upload a file
#[utoipa::path(
    post,
    path = "/api/v1/media",
    responses(
        (status = 201, description = "File uploaded successfully"),
        (status = 400, description = "No file uploaded")
    ),
    tag = "media",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upload_file(
    req: web::HttpRequest,
    mut payload: Multipart,
    service: web::types::State<Arc<MediaService>>,
    claims: Claims,
    audit: web::types::State<Arc<AuditService>>,
) -> web::HttpResponse {
    let tenant_id = claims.tenant_id;
    
    let mut filename = String::new();
    let mut original_name = String::new();
    let mut mime_type = String::new();
    let mut file_size: i64 = 0;
    
    let query = web::types::Query::<AssetQuery>::from_query(req.query_string()).ok();
    let folder_id = query.and_then(|q| q.folder_id);

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return web::HttpResponse::BadRequest().json(&serde_json::json!({"error": "Invalid file"}))
        };
        
        original_name = field.headers()
            .get("content-disposition")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split(';')
                    .find(|p| p.trim().starts_with("filename="))
                    .map(|p| p.trim().trim_start_matches("filename=").trim_matches('"'))
            })
            .unwrap_or("unknown")
            .to_string();

        mime_type = field.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let ext = Path::new(&original_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        
        let id = Uuid::new_v4();
        filename = format!("{}.{}", id, ext);
        
        let tenant_dir = format!("./uploads/{}", tenant_id);
        if std::fs::create_dir_all(&tenant_dir).is_err() {
            return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Storage error"}));
        }
        
        let filepath = format!("{}/{}", tenant_dir, filename);
        
        let mut f = match std::fs::File::create(&filepath) {
            Ok(file) => file,
            Err(_) => return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "File creation error"}))
        };
        
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => {
                    file_size += data.len() as i64;
                    if std::io::copy(&mut data.as_ref(), &mut f).is_err() {
                        return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Write error"}));
                    }
                },
                Err(_) => return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Chunk error"}))
            }
        }
    }

    if filename.is_empty() {
        return web::HttpResponse::BadRequest().json(&serde_json::json!({"error": "No file uploaded"}));
    }

    let url = format!("/api/v1/media/{}/{}", tenant_id, filename);
    match service.create_asset(tenant_id, folder_id, filename.clone(), original_name, mime_type, file_size, url).await {
        Ok(asset) => {
            let _ = audit.log(&tenant_id.to_string(), "MEDIA_UPLOADED", Some(&filename), "SUCCESS", None).await;
            web::HttpResponse::Created().json(&asset)
        },
        Err(_) => web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Database error"}))
    }
}

/// Create a folder
#[utoipa::path(
    post,
    path = "/api/v1/media/folders",
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created successfully"),
        (status = 400, description = "Invalid request")
    ),
    tag = "media",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_folder(
    body: web::types::Json<CreateFolderRequest>,
    service: web::types::State<Arc<MediaService>>,
    claims: Claims,
) -> web::HttpResponse {
    let tenant_id = claims.tenant_id;
    
    match service.create_folder(tenant_id, body.parent_id, body.name.clone()).await {
        Ok(folder) => web::HttpResponse::Created().json(&folder),
        Err(_) => web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Failed to create folder"}))
    }
}

/// List assets and folders
#[utoipa::path(
    get,
    path = "/api/v1/media/assets",
    params(
        AssetQuery
    ),
    responses(
        (status = 200, description = "List of assets and folders")
    ),
    tag = "media",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_assets(
    query: web::types::Query<AssetQuery>,
    service: web::types::State<Arc<MediaService>>,
    claims: Claims,
) -> web::HttpResponse {
    let tenant_id = claims.tenant_id;
    
    let folders = match service.list_folders(tenant_id, query.folder_id).await {
        Ok(f) => f,
        Err(_) => return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Failed to list folders"}))
    };
        
    let assets = match service.list_assets(tenant_id, query.folder_id).await {
        Ok(a) => a,
        Err(_) => return web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Failed to list assets"}))
    };
        
    web::HttpResponse::Ok().json(&serde_json::json!({
        "folders": folders,
        "assets": assets
    }))
}

/// Serve an uploaded file
#[utoipa::path(
    get,
    path = "/api/v1/media/{tenant_id}/{filename}",
    responses(
        (status = 200, description = "File served successfully"),
        (status = 404, description = "File not found")
    ),
    tag = "media"
)]
pub async fn serve_file(
    path: web::types::Path<(Uuid, String)>,
) -> Result<ntex_files::NamedFile, web::Error> {
    let (tenant_id, filename) = path.into_inner();
    let filepath = format!("./uploads/{}/{}", tenant_id, filename);
    
    match ntex_files::NamedFile::open(&filepath) {
        Ok(file) => Ok(file),
        Err(e) => Err(web::Error::from(e))
    }
}

/// Delete an asset
#[utoipa::path(
    delete,
    path = "/api/v1/media/assets/{id}",
    responses(
        (status = 200, description = "Asset deleted successfully"),
        (status = 404, description = "Asset not found")
    ),
    tag = "media",
    params(
        ("id" = Uuid, Path, description = "Asset ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_asset(
    path: web::types::Path<(Uuid,)>,
    service: web::types::State<Arc<MediaService>>,
    claims: Claims,
    audit: web::types::State<Arc<AuditService>>,
) -> web::HttpResponse {
    let tenant_id = claims.tenant_id;
    let asset_id = path.into_inner().0;
    
    match service.delete_asset(asset_id, tenant_id).await {
        Ok(_) => {
            let _ = audit.log(&tenant_id.to_string(), "MEDIA_DELETED", Some(&asset_id.to_string()), "SUCCESS", None).await;
            web::HttpResponse::Ok().json(&serde_json::json!({ "success": true }))
        },
        Err(_) => web::HttpResponse::InternalServerError().json(&serde_json::json!({"error": "Failed to delete"}))
    }
}
