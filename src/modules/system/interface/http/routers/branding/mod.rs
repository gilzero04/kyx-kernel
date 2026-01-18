use super::super::handlers::branding;
use ntex::web;

/// Public branding routes
pub fn branding_public_routes() -> web::Scope<ntex::web::DefaultError> {
    web::scope("/branding").route("", web::get().to(branding::get_branding))
}
