//! Authentication: the frontend logs in with the shared password and receives a
//! JWT, which it sends as `authorization: Bearer <token>` metadata on every
//! subsequent RPC. Workers authenticate separately with a static shared secret
//! (`[backend].worker_token`).

use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

lazy_static! {
    pub static ref BACKEND_PASSWORD: Mutex<String> = Mutex::new(String::new());
    pub static ref BACKEND_SECRET: Mutex<String> = Mutex::new(String::new());
    pub static ref WORKER_TOKEN: Mutex<Option<String>> = Mutex::new(None);
    static ref RATE_LIMIT: Mutex<HashMap<String, RateLimitEntry>> = Mutex::new(HashMap::new());
}

/// When true, workers are accepted without a token (only reachable if the
/// operator explicitly set `allow_insecure_workers` and left `worker_token`
/// unset — otherwise the backend refuses to start). Defaults to fail-closed.
pub static ALLOW_INSECURE_WORKERS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}

pub fn jwt_encode(claims: &Claims) -> Result<String, Status> {
    let secret = BACKEND_SECRET
        .lock()
        .expect("secret mutex poisoned")
        .clone();
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| Status::internal("failed to encode token"))
}

pub fn jwt_decode(token: &str) -> Result<Claims, ()> {
    let secret = BACKEND_SECRET
        .lock()
        .expect("secret mutex poisoned")
        .clone();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ())
}

/// Pulls a Bearer token out of the `authorization` metadata, if present.
fn bearer<T>(req: &Request<T>) -> Option<String> {
    let raw = req.metadata().get("authorization")?.to_str().ok()?;
    raw.strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .map(|s| s.to_string())
}

/// Rejects the request unless it carries a valid session JWT. Used by every API
/// RPC except `Login`/`Me`-style probes that define their own behaviour.
pub fn require_session<T>(req: &Request<T>) -> Result<(), Status> {
    let token = bearer(req).ok_or_else(|| Status::unauthenticated("missing token"))?;
    jwt_decode(&token).map_err(|_| Status::unauthenticated("invalid token"))?;
    Ok(())
}

/// Validates a worker's shared-secret token. Fails closed: when no token is
/// configured the request is rejected unless the operator explicitly opted into
/// insecure workers (see [`ALLOW_INSECURE_WORKERS`]).
pub fn require_worker_token<T>(req: &Request<T>) -> Result<(), Status> {
    let expected = WORKER_TOKEN
        .lock()
        .expect("worker token mutex poisoned")
        .clone();
    let Some(expected) = expected else {
        if ALLOW_INSECURE_WORKERS.load(Ordering::Relaxed) {
            return Ok(());
        }
        return Err(Status::unauthenticated("worker authentication required"));
    };
    let token = bearer(req).ok_or_else(|| Status::unauthenticated("missing worker token"))?;
    if token == expected {
        Ok(())
    } else {
        Err(Status::unauthenticated("invalid worker token"))
    }
}

// ---------------------------------------------------------------------------
// Login rate limiting (per source IP, best-effort: gRPC-web is behind nginx).
// ---------------------------------------------------------------------------

const MAX_ATTEMPTS: u32 = 5;
const WINDOW: Duration = Duration::from_secs(60);

struct RateLimitEntry {
    attempts: u32,
    window_start: Instant,
}

pub fn check_rate_limit(ip: &str) -> bool {
    let mut map = RATE_LIMIT.lock().expect("rate limit mutex poisoned");
    let now = Instant::now();
    map.retain(|_, entry| now.duration_since(entry.window_start) < WINDOW);

    let entry = map.entry(ip.to_owned()).or_insert(RateLimitEntry {
        attempts: 0,
        window_start: now,
    });
    if now.duration_since(entry.window_start) >= WINDOW {
        entry.attempts = 0;
        entry.window_start = now;
    }
    entry.attempts += 1;
    entry.attempts <= MAX_ATTEMPTS
}

pub fn generate_random_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
