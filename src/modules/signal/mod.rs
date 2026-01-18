// Signal Module for kyx-kernel
// Handles signal token issuance for kyx-signal integration

pub mod application;
pub mod domain;
pub mod interface;

use ntex::web;
use std::sync::Arc;

use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::redis::Redis;
use crate::core::utils::jwt::JwtService;

pub struct SignalModule {
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    redis: Option<Arc<Redis>>,
}

impl SignalModule {
    pub fn new(jwt: Arc<JwtService>, audit: Arc<AuditService>, redis: Option<Arc<Redis>>) -> Self {
        Self { jwt, audit, redis }
    }

    pub fn configure(&self, config: &mut web::ServiceConfig) {
        let jwt = self.jwt.clone();
        let audit = self.audit.clone();
        let redis = self.redis.clone();

        config.service(
            web::scope("/signal")
                .state(jwt.clone())
                .state(audit.clone())
                .configure(|cfg| interface::http::routers::signal_routes(cfg, jwt, audit, redis)),
        );
    }
}
