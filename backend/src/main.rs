use std::{env, sync::Arc};

use api::{get_players::get_players, get_server::get_server, get_server_range::get_server_range};
use axum::{middleware, routing::post, Json, Router};
use database::DatabaseWrapper;
use serde_json::{json, Value};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::Level;

mod api;
mod auth;
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

    let protected = Router::new()
        .route("/api/server", post(get_server))
        .route("/api/servers", post(get_server_range))
        .route("/api/players", post(get_players))
        .route("/api/check", post(check_password))
        .layer(middleware::from_fn(auth::password_middleware));

    let app = Router::new()
        .route("/api/protected", post(is_protected))
        .merge(protected)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(logging)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    if env::var("BACKEND_PASSWORD").is_ok() {
        println!("[+] Backend Password is set");
    }

    println!("🚀 Server started successfully");
    axum::serve(listener, app).await.unwrap();
}

async fn is_protected() -> Json<Value> {
    Json(json!(env::var("BACKEND_PASSWORD").is_ok()))
}

async fn check_password() {}
