use std::{env, sync::Arc};

use api::{get_players::get_players, get_server::get_server, get_server_range::get_server_range};
use auth::{auth_api, check_password};
use axum::{middleware, routing::post, Router};
use database::DatabaseWrapper;
use rand::{distr::Alphanumeric, rng, Rng};
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
        .route("/server", post(get_server))
        .route("/servers", post(get_server_range))
        .route("/players", post(get_players))
        .route("/check_auth", post(check_password))
        .layer(middleware::from_fn(auth::auth_middleware));

    let api = Router::new()
        .route("/auth", post(auth_api))
        .merge(protected);

    let app = Router::new()
        .nest("/api", api)
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
