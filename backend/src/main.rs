use std::{env, sync::Arc};

use api::{
    auth::{authenticate_user, validate_credentials},
    fetch_players::fetch_player_list,
    fetch_server_info::fetch_server_info,
    fetch_server_list::fetch_server_list,
    fetch_stats::fetch_stats,
};
use axum::{middleware, routing::post, Router};
use database::DatabaseWrapper;
use rand::{distr::Alphanumeric, rng, Rng};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::Level;

mod api;
mod api_middleware;
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

    let protected_routes = Router::new()
        .route("/server/info", post(fetch_server_info))
        .route("/servers/list", post(fetch_server_list))
        .route("/players/list", post(fetch_player_list))
        .route("/auth/validate", post(validate_credentials))
        .route("/stats", post(fetch_stats))
        .layer(middleware::from_fn(api_middleware::middleware_check));

    let public_api = Router::new()
        .route("/auth/login", post(authenticate_user))
        .merge(protected_routes);

    let app = Router::new()
        .nest("/api/v1", public_api)
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
    } else {
        let password = generate_random_string(8);
        env::set_var("BACKEND_PASSWORD", &password);
        println!("[+] Backend password: {}", password);
    }

    let secret = generate_random_string(32);
    env::set_var("BACKEND_SECRET", &secret);
    println!("[+] Secret is set");

    println!("🚀 Server started successfully");
    axum::serve(listener, app).await.unwrap();
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char) // генерируем случайные символы
        .collect() // собираем в строку
}
