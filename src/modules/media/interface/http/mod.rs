use ntex::web;
use ntex_multipart::Multipart;
use futures_util::StreamExt;
use uuid::Uuid;
use crate::core::infrastructure::audit::AuditService;
use std::sync::Arc;
use std::path::Path;

#[web::post("")]
pub async fn upload_file(
    mut payload: Multipart,
    audit: web::types::State<Arc<AuditService>>,
) -> Result<web::HttpResponse, web::Error> {
    let mut filename = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| web::Error::from(e))?;
        
        let filename_attr = field.headers()
            .get("content-disposition")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split(';')
                    .find(|p| p.trim().starts_with("filename="))
                    .map(|p| p.trim().trim_start_matches("filename=").trim_matches('"'))
            });

        if let Some(name) = filename_attr {
            let ext = Path::new(name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("bin");
            
            let id = Uuid::new_v4();
            filename = format!("{}.{}", id, ext);
            let filepath = format!("./uploads/{}", filename);
            
            let mut f = std::fs::File::create(&filepath)?;
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| web::Error::from(e))?;
                std::io::copy(&mut data.as_ref(), &mut f).map_err(|e| web::Error::from(e))?;
            }
        }
    }

    if filename.is_empty() {
        return Ok(web::HttpResponse::BadRequest().json(&serde_json::json!({
            "error": "No file uploaded"
        })));
    }

    let _ = audit.log("SuperAdmin", "MEDIA_UPLOADED", Some(&filename), "SUCCESS", None).await;

    Ok(web::HttpResponse::Created().json(&serde_json::json!({
        "status": "success",
        "filename": filename,
        "url": format!("/api/v1/media/{}", filename)
    })))
}

#[web::get("/{filename}")]
pub async fn serve_file(
    path: web::types::Path<(String,)>,
) -> Result<ntex_files::NamedFile, web::Error> {
    let filename = &path.0;
    let filepath = format!("./uploads/{}", filename);
    
    let file = ntex_files::NamedFile::open(filepath)?;
    Ok(file)
}
