use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use db_schema::{models::scan_targets::TargetInsert, schema};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use ts_rs::TS;

use crate::{database::DatabaseWrapper, error::AppError};

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct AddAddrRequest {
    pub addr: String,
    pub quick: bool,
}

const DEFAULT_PORT: i32 = 25565;

fn parse_port(s: &str) -> Result<i32, AppError> {
    s.parse::<i32>()
        .map_err(|_| AppError::bad_request(format!("Invalid port: {s}")))
}

impl AddAddrRequest {
    /// Splits the address into `(ip, port)`, supporting:
    /// - `host` / `1.2.3.4`            -> default port
    /// - `host:port` / `1.2.3.4:port`  -> explicit port (exactly one colon)
    /// - `[::1]` / `[::1]:port`         -> bracketed IPv6, optional port
    /// - `::1` / `2001:db8::1`          -> bare IPv6 (multiple colons), default port
    pub fn to_target_insert<'a>(&'a self) -> Result<TargetInsert<'a>, AppError> {
        let addr = self.addr.trim();

        let (ip, port) = if let Some(rest) = addr.strip_prefix('[') {
            // Bracketed IPv6: "[::1]" or "[::1]:25565"
            let (ip, after) = rest
                .split_once(']')
                .ok_or_else(|| AppError::bad_request(format!("Invalid address: {addr}")))?;
            let port = match after.strip_prefix(':') {
                Some(p) => parse_port(p)?,
                None if after.is_empty() => DEFAULT_PORT,
                None => return Err(AppError::bad_request(format!("Invalid address: {addr}"))),
            };
            (ip, port)
        } else if addr.matches(':').count() == 1 {
            // Exactly one colon: host:port (IPv4 or hostname).
            let (ip, port_str) = addr.rsplit_once(':').unwrap();
            (ip, parse_port(port_str)?)
        } else {
            // No colon (default port) or multiple colons (bare IPv6) -> use as-is.
            (addr, DEFAULT_PORT)
        };

        Ok(TargetInsert {
            ip,
            port,
            quick: self.quick,
        })
    }
}

pub async fn add_target(
    State(db): State<Arc<DatabaseWrapper>>,
    Json(body): Json<AddAddrRequest>,
) -> Result<StatusCode, AppError> {
    let target = body.to_target_insert()?;

    let mut conn = db
        .pool
        .get()
        .await
        .map_err(|e| AppError::db("Failed to acquire DB connection in add_target", e))?;

    insert_into(schema::scan_targets::table)
        .values(&target)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::db("Failed to insert target", e))?;

    Ok(StatusCode::OK)
}
