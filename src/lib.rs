use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

pub mod error;
pub mod handlers;
pub mod models;
pub mod store;

pub use models::Todo;
pub use store::JsonStore;

pub fn create_app(store: JsonStore) -> Router {
    Router::new()
        .route("/", get(handlers::index))
        .route("/health", get(handlers::health))
        .route(
            "/todos",
            get(handlers::list_todos).post(handlers::create_todo),
        )
        .route(
            "/todos/{id}",
            get(handlers::get_todo)
                .put(handlers::update_todo)
                .delete(handlers::delete_todo),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(store)
}
