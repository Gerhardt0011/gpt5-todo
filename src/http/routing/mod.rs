pub mod todos;

use axum::{routing::get, Router};

pub fn app(router: Router) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(router)
}
