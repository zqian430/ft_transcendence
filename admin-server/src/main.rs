use axum::{routing::get, Router};
use std::env;

#[tokio::main]
async fn main() {
    // Read the port from .env (GAME_SERVER_PORT or ADMIN_SERVER_PORT)
    let port = env::var("ADMIN_SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    let addr = format!("0.0.0.0:{}", port);
    println!("Serveur démarré sur {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}