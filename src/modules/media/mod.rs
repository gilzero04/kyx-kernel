use ntex::web;
use crate::core::AppModule;
use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::database::Database;
use std::sync::Arc;

pub mod interface;

pub struct MediaModule {
    _db: Arc<Database>,
    audit: Arc<AuditService>,
}

impl MediaModule {
    pub fn new(db: Arc<Database>, audit: Arc<AuditService>) -> Self {
        Self { _db: db, audit }
    }
}

impl AppModule for MediaModule {
    fn name(&self) -> &str {
        "media"
    }

    fn try_configure(&self, config: &mut web::ServiceConfig) -> Result<(), crate::core::AppError> {
        let audit_service = self.audit.clone();
        
        config.service(
            web::scope("/media")
                .state(audit_service)
                .route("", web::post().to(interface::http::handlers::media::upload_file))
                .route("/{filename}", web::get().to(interface::http::handlers::media::serve_file))
        );

        Ok(())
    }
}
