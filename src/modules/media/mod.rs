use crate::core::AppModule;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::database::Database;
use ntex::web;
use std::sync::Arc;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

use crate::core::utils::jwt::JwtService;
use crate::modules::media::application::services::media::MediaService;
use crate::modules::media::infrastructure::repositories::media::PostgresMediaRepository;

pub struct MediaModule {
    #[allow(dead_code)]
    db: Arc<Database>,
    #[allow(dead_code)]
    audit: Arc<AuditService>,
    service: Arc<MediaService>,
    jwt: Arc<JwtService>,
}

impl MediaModule {
    pub fn new(db: Arc<Database>, audit: Arc<AuditService>, jwt: Arc<JwtService>) -> Self {
        let repo = Arc::new(PostgresMediaRepository::new(db.clone()));
        let service = Arc::new(MediaService::new(repo));
        Self {
            db,
            audit,
            service,
            jwt,
        }
    }
}

impl AppModule for MediaModule {
    fn name(&self) -> &str {
        "media"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let audit_service = self.audit.clone();
        let media_service = self.service.clone();
        let jwt_service = self.jwt.clone();

        use crate::core::infrastructure::permission_middleware::RequirePermission;

        config.service(
            web::scope("/media")
                .state(audit_service.clone())
                .state(media_service)
                .state(jwt_service.clone())
                .service(
                    web::resource("")
                        .guard(web::guard::Post())
                        .wrap(RequirePermission::new(
                            "media:upload",
                            jwt_service.clone(),
                            audit_service.clone(),
                        ))
                        .route(web::post().to(interface::http::handlers::media::upload_file)),
                )
                .service(
                    web::resource("")
                        .guard(web::guard::Get())
                        .wrap(RequirePermission::new(
                            "media:read",
                            jwt_service.clone(),
                            audit_service.clone(),
                        ))
                        .route(web::get().to(interface::http::handlers::media::list_assets)),
                )
                .service(
                    web::resource("/{id}")
                        .guard(web::guard::Delete())
                        .wrap(RequirePermission::new(
                            "media:delete",
                            jwt_service.clone(),
                            audit_service.clone(),
                        ))
                        .route(web::delete().to(interface::http::handlers::media::delete_asset)),
                )
                .service(
                    web::resource("/folders")
                        .guard(web::guard::Post())
                        .wrap(RequirePermission::new(
                            "media:folder",
                            jwt_service.clone(),
                            audit_service.clone(),
                        ))
                        .route(web::post().to(interface::http::handlers::media::create_folder)),
                )
                .route(
                    "/{tenant_id}/{filename}",
                    web::get().to(interface::http::handlers::media::serve_file),
                ),
        );

        Ok(())
    }
}
