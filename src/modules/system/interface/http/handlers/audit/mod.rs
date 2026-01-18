use crate::core::utils::response::ApiResponse;
use crate::modules::system::application::services::audit::AuditQueryService;
use crate::modules::system::domain::audit::AuditLogFilter;
use crate::modules::system::interface::http::dto::audit::LogsQuery;
use ntex::web;
use serde_json::json;
use std::sync::Arc;

/// List audit logs (Admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/logs",
    params(
        LogsQuery
    ),
    responses(
        (status = 200, description = "List of audit logs", body = Vec<AuditLogEntry>)
    ),
    tag = "audit",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_audit_logs(
    service: web::types::State<Arc<AuditQueryService>>,
    query: web::types::Query<LogsQuery>,
) -> impl web::Responder {
    let filter = AuditLogFilter {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(50),
        sort: query.sort.clone(),
        search: query.search.clone(),
    };

    match service.list_logs(filter).await {
        Ok(data) => {
            let response =
                ApiResponse::ok(json!({ "logs": data }), "Audit logs listed successfully");
            web::HttpResponse::Ok().json(&response)
        }
        Err(e) => {
            let response =
                ApiResponse::<()>::internal_error(&format!("Failed to fetch logs: {}", e));
            web::HttpResponse::InternalServerError().json(&response)
        }
    }
}
