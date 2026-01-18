use crate::modules::system::interface::http::handlers::share;
use ntex::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Resource Sharing endpoints
    cfg.service(
        web::scope("/shares")
            .route("", web::get().to(share::list_shares))
            .route("", web::post().to(share::create_share))
            .route("/received", web::get().to(share::list_received_shares))
            .route("/{id}", web::delete().to(share::revoke_share))
            .route("/{id}/usage", web::get().to(share::get_share_usage)),
    );
}
