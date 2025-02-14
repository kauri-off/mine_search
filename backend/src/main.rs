use std::sync::Arc;

use api::{get_players::get_players, get_server::get_server, get_server_range::get_server_range};
use axum::{routing::post, Router};
use database::DatabaseWrapper;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::Level;

mod api;
mod database;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let logging = TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO));

    let db = Arc::new(DatabaseWrapper::establish());

    let app = Router::new()
        .route("/api/server", post(get_server))
        .route("/api/servers", post(get_server_range))
        .route("/api/players", post(get_players))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(logging)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Server started successfully");
    axum::serve(listener, app).await.unwrap();
}
