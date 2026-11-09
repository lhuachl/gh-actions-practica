use practica::{JsonStore, create_app};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let db_path = std::env::var("TODO_DB_PATH").unwrap_or_else(|_| "todos.json".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let store = JsonStore::new(db_path).expect("failed to init store");
    let app = create_app(store);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}", port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}
