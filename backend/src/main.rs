use std::{env, sync::Arc};

use api::{
    add_ip::add_ip, add_ips::add_ips, auth::authenticate_user,
    fetch_server_data::fetch_server_data, fetch_server_info::fetch_server_info,
    fetch_server_list::fetch_server_list, fetch_stats::fetch_stats, server_delete::server_delete,
    update_server::update_server,
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        Method,
        header::{CONTENT_TYPE, COOKIE},
    },
    middleware,
    routing::post,
};
use database::DatabaseWrapper;
use lazy_static::lazy_static;
use rand::{RngExt, distr::Alphanumeric};
use tokio::sync::Mutex;
use tower_http::{
    cors::CorsLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;

use crate::api::{
    fetch_players_list::fetch_players_list, ping_server::ping_server, update_player::update_player,
};

lazy_static! {
    static ref BACKEND_PASSWORD: Mutex<String> = Mutex::new(String::new());
    static ref BACKEND_SECRET: Mutex<String> = Mutex::new(String::new());
}

mod api;
mod api_middleware;
mod database;
mod error;
mod jwt_wrapper;

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

    let protected_routes = Router::new()
        .route("/server/info", post(fetch_server_info))
        .route("/server/update", post(update_server))
        .route("/server/data", post(fetch_server_data))
        .route("/server/list", post(fetch_server_list))
        .route("/server/delete", post(server_delete))
        .route("/server/ping", post(ping_server))
        .route("/player/list", post(fetch_players_list))
        .route("/player/update", post(update_player))
        .route("/ip/add", post(add_ip))
        .route("/ip/add_list", post(add_ips))
        .route("/stats", post(fetch_stats))
        .layer(middleware::from_fn(api_middleware::middleware_check))
        .layer(DefaultBodyLimit::disable());

    let public_api = Router::new()
        .route("/auth/login", post(authenticate_user))
        .merge(protected_routes);

    let app = Router::new()
        .nest("/api/v1", public_api)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers([COOKIE, CONTENT_TYPE])
                .allow_credentials(true),
        )
        .layer(logging)
        .with_state(db);

    match env::var("BACKEND_PASSWORD") {
        Ok(t) => {
            let mut password_mutex = BACKEND_PASSWORD.lock().await;
            let mut secret_mutex = BACKEND_SECRET.lock().await;
            *password_mutex = t;
            *secret_mutex = env::var("BACKEND_JWT_SECRET").unwrap_or_else(|_| generate_random_string(32));
        }
        Err(_) => {
            eprintln!("[-] You must set BACKEND_PASSWORD in .env");
            return;
        }
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("Server started on 0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server crashed");
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
