use ntex::web;
use crate::modules::system::interface::http::handlers::tenant;
use ntex::web::DefaultError;

pub fn tenant_routes() -> web::Scope<DefaultError> {
    web::scope("/tenants")
        .route("", web::get().to(tenant::list_tenants))
        .route("/owner", web::patch().to(tenant::update_owner))
}
