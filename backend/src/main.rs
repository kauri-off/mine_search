use std::{env, sync::Arc};

use api::{
    auth::{authenticate_user, validate_credentials},
    fetch_server_info::fetch_server_info,
    fetch_server_list::fetch_server_list,
    fetch_stats::fetch_stats,
    set_cookie::set_cookie,
    update_server::update_server,
};
use axum::{
    Router,
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

lazy_static! {
    static ref BACKEND_PASSWORD: Mutex<String> = Mutex::new(String::new());
    static ref BACKEND_SECRET: Mutex<String> = Mutex::new(String::new());
}

mod api;
mod api_middleware;
mod database;
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
        .route("/servers/list", post(fetch_server_list))
        .route("/auth/validate", post(validate_credentials))
        .route("/stats", post(fetch_stats))
        .layer(middleware::from_fn(api_middleware::middleware_check));

    let public_api = Router::new()
        .route("/auth/login", post(authenticate_user))
        .route("/auth/set_cookie", post(set_cookie))
        .merge(protected_routes);

    let app = Router::new()
        .nest("/api/v1", public_api)
        .layer(
            CorsLayer::new()
                //     .allow_origin([
                //         "http://localhost:8080".parse().unwrap(),
                //         "http://127.0.0.1:8080".parse().unwrap(),
                //     ])
                //     .allow_methods([
                //         Method::GET,
                //         Method::POST,
                //         Method::PUT,
                //         Method::DELETE,
                //         Method::PATCH,
                //         Method::OPTIONS,
                //         Method::HEAD,
                //     ])
                //     .allow_headers([
                //         "content-type".parse().unwrap(),
                //         "authorization".parse().unwrap(),
                //         "cookie".parse().unwrap(),
                //         "accept".parse().unwrap(),
                //         "x-requested-with".parse().unwrap(),
                //     ])
                //     .expose_headers([
                //         "content-type".parse().unwrap(),
                //         "content-length".parse().unwrap(),
                //         "set-cookie".parse().unwrap(),
                //         "authorization".parse().unwrap(),
                //     ])
                //     .allow_credentials(true)
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers([COOKIE, CONTENT_TYPE])
                .allow_credentials(true),
        )
        .layer(logging)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    match env::var("BACKEND_PASSWORD") {
        Ok(t) => {
            let mut password_mutex = BACKEND_PASSWORD.lock().await;
            let mut secret_mutex = BACKEND_SECRET.lock().await;
            *password_mutex = t;
            *secret_mutex = generate_random_string(32);
        }
        Err(_) => {
            eprintln!("[-] You must set BACKEND_PASSWORD in .env");
            return;
        }
    }

    println!("Server started successfully");
    axum::serve(listener, app).await.unwrap();
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
