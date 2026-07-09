//! Database writes for results streamed in from workers. The backend is the
//! sole writer and workers only report structured outcomes.
//!
//! Delivery from workers is at-least-once (they replay un-acked results on
//! reconnect / a periodic sweep), so every write is made idempotent: each
//! result carries a stable `result_id`, recorded in `processed_results` as the
//! first statement of the write transaction. A result whose id is already
//! present is a replay and is skipped (returning `Ok(None)`), which keeps the
//! append-only `player_count_snapshots` from accumulating duplicates. All
//! statements for one result run on a single connection inside one transaction,
//! so a mid-write failure rolls back cleanly with no partial state.

use std::time::Duration;

use crate::{
    chat::ChatObject,
    models::{
        player_count_snapshots::SnapshotInsert,
        players::PlayerInsert,
        servers::{
            JoinStatus, ServerExtraUpdate, ServerInsert, ServerModel, ServerModelMini, ServerUpdate,
        },
    },
    schema,
};
use chrono::Utc;
use diesel::{dsl::insert_into, pg::Pg, prelude::*, sql_types::Bool};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use proto::worker::ServerReport;

use crate::database::DatabaseWrapper;

pub type DbResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Keeps only the 100 newest snapshots for a server.
const PRUNE_SQL: &str = "DELETE FROM player_count_snapshots \
     WHERE server_id = $1 \
     AND recorded_at < ( \
         SELECT recorded_at FROM player_count_snapshots \
         WHERE server_id = $1 \
         ORDER BY recorded_at DESC \
         LIMIT 1 OFFSET 99 \
     )";

/// How long to keep `processed_results` idempotency rows. Must comfortably
/// exceed the worker's outbox replay horizon so a replayed result is always
/// recognised as a duplicate. Kept in sync with the worker's outbox max age.
const PROCESSED_RETENTION_HOURS: i64 = 24;

/// Per-write retry policy for transient DB errors (connection dropped, pool
/// timeout). Safe because the transaction is atomic and idempotent.
const RETRY_ATTEMPTS: usize = 3;
const RETRY_BASE_DELAY: Duration = Duration::from_millis(100);

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
}

/// Flattens a server description JSON into plaintext MOTD for the queryable
/// `motd` column. Returns an empty string when the JSON isn't a chat component.
fn motd_from_description(description: &serde_json::Value) -> String {
    serde_json::from_value::<ChatObject>(description.clone())
        .map(|chat| chat.get_motd())
        .unwrap_or_default()
}

/// Runs `op` up to [`RETRY_ATTEMPTS`] times with exponential backoff. Intended
/// for whole write transactions, which are idempotent and so safe to retry.
async fn with_retry<F, Fut, T>(mut op: F) -> DbResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = DbResult<T>>,
{
    let mut delay = RETRY_BASE_DELAY;
    for attempt in 1..=RETRY_ATTEMPTS {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt < RETRY_ATTEMPTS => {
                tracing::warn!(
                    "db write failed (attempt {attempt}/{RETRY_ATTEMPTS}): {e}; retrying"
                );
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!("loop returns on the final attempt")
}

/// Records `result_id` in the idempotency ledger. Returns `true` if this is the
/// first time we have seen it (caller should apply the write), `false` if it is
/// a replay (caller should skip). Run as the first statement of the write txn.
async fn claim_result(conn: &mut AsyncPgConnection, result_id: &str) -> QueryResult<bool> {
    // An empty id (no outbox / legacy worker) opts out of idempotency: always
    // apply, and never write `""` to the ledger (which would poison all future
    // empty-id results into being skipped as "replays").
    if result_id.is_empty() {
        return Ok(true);
    }
    let inserted = insert_into(schema::processed_results::table)
        .values(schema::processed_results::result_id.eq(result_id))
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;
    Ok(inserted == 1)
}

/// Upsert-by-ip; conflict only bumps updated_at/is_online/favicon (rediscovery
/// does not overwrite version/description).
pub async fn persist_discovered(
    db: &DatabaseWrapper,
    report: ServerReport,
    result_id: &str,
) -> DbResult<Option<i32>> {
    with_retry(|| discovered_txn(db, &report, result_id)).await
}

async fn discovered_txn(
    db: &DatabaseWrapper,
    report: &ServerReport,
    result_id: &str,
) -> DbResult<Option<i32>> {
    let mut conn = db.conn().await?;
    let conn: &mut AsyncPgConnection = &mut conn;
    let server_id = conn
        .transaction::<Option<i32>, diesel::result::Error, _>(async |conn| {
            if !claim_result(conn, result_id).await? {
                return Ok(None); // replay — already persisted
            }

            let description = parse_json(&report.description_json);
            let motd = motd_from_description(&description);
            let (is_online_mode, disconnect_reason) = match &report.extra {
                Some(e) => (
                    e.is_online_mode,
                    e.disconnect_reason_json.as_deref().map(parse_json),
                ),
                None => (false, None),
            };
            let favicon = report.favicon.as_deref();

            let server_insert = ServerInsert {
                ip: &report.ip,
                port: report.port,
                version_name: &report.version_name,
                protocol: report.protocol,
                description: &description,
                motd: &motd,
                is_online_mode,
                disconnect_reason,
                requires_mods: report.requires_mods,
                favicon,
                ping: report.ping,
            };

            let server: ServerModel = insert_into(schema::servers::table)
                .values(&server_insert)
                .on_conflict(schema::servers::ip)
                .do_update()
                .set((
                    schema::servers::updated_at.eq(Utc::now()),
                    schema::servers::is_online.eq(true),
                    schema::servers::favicon.eq(favicon),
                ))
                .returning(ServerModel::as_returning())
                .get_result(conn)
                .await?;

            write_snapshot_and_players(conn, server.id, report, false).await?;
            Ok(Some(server.id))
        })
        .await?;
    Ok(server_id)
}

/// Full field update by ip (reachable path).
pub async fn persist_updated(
    db: &DatabaseWrapper,
    report: ServerReport,
    result_id: &str,
) -> DbResult<Option<i32>> {
    with_retry(|| updated_txn(db, &report, result_id)).await
}

async fn updated_txn(
    db: &DatabaseWrapper,
    report: &ServerReport,
    result_id: &str,
) -> DbResult<Option<i32>> {
    let mut conn = db.conn().await?;
    let conn: &mut AsyncPgConnection = &mut conn;
    let server_id = conn
        .transaction::<Option<i32>, diesel::result::Error, _>(async |conn| {
            if !claim_result(conn, result_id).await? {
                return Ok(None); // replay — already persisted
            }

            let Some(server_id) = schema::servers::table
                .filter(schema::servers::ip.eq(&report.ip))
                .select(schema::servers::id)
                .first::<i32>(conn)
                .await
                .optional()?
            else {
                // Server vanished between scheduling and reporting.
                return Ok(None);
            };

            let description = parse_json(&report.description_json);
            let motd = motd_from_description(&description);
            let favicon = report.favicon.as_deref();

            let server_change = ServerUpdate {
                version_name: &report.version_name,
                protocol: report.protocol,
                description: &description,
                motd: &motd,
                updated_at: Utc::now(),
                is_online: true,
                requires_mods: report.requires_mods,
                favicon,
                ping: report.ping,
            };

            diesel::update(schema::servers::table)
                .filter(schema::servers::id.eq(server_id))
                .set(server_change)
                .execute(conn)
                .await?;

            if let Some(extra) = &report.extra {
                let server_extra_change = ServerExtraUpdate {
                    is_online_mode: extra.is_online_mode,
                    disconnect_reason: extra.disconnect_reason_json.as_deref().map(parse_json),
                };
                diesel::update(schema::servers::table)
                    .filter(schema::servers::id.eq(server_id))
                    .set(server_extra_change)
                    .execute(conn)
                    .await?;
            }

            write_snapshot_and_players(conn, server_id, report, true).await?;
            Ok(Some(server_id))
        })
        .await?;
    Ok(server_id)
}

/// Marks a server offline after a failed re-probe.
pub async fn persist_offline(
    db: &DatabaseWrapper,
    ip: &str,
    result_id: &str,
) -> DbResult<Option<i32>> {
    with_retry(|| offline_txn(db, ip, result_id)).await
}

async fn offline_txn(db: &DatabaseWrapper, ip: &str, result_id: &str) -> DbResult<Option<i32>> {
    let mut conn = db.conn().await?;
    let conn: &mut AsyncPgConnection = &mut conn;
    let id = conn
        .transaction::<Option<i32>, diesel::result::Error, _>(async |conn| {
            if !claim_result(conn, result_id).await? {
                return Ok(None); // replay — already persisted
            }
            diesel::update(schema::servers::table)
                .filter(schema::servers::ip.eq(ip))
                .set(schema::servers::is_online.eq(false))
                .returning(schema::servers::id)
                .get_result::<i32>(conn)
                .await
                .optional()
        })
        .await?;
    Ok(id)
}

/// Inserts a player-count snapshot, prunes old ones, and records players. When
/// `update_last_seen` is set, existing players have their `last_seen_at` bumped
/// (update cycle); otherwise duplicates are ignored (discovery). Runs on the
/// caller's transaction connection.
async fn write_snapshot_and_players(
    conn: &mut AsyncPgConnection,
    server_id: i32,
    report: &ServerReport,
    update_last_seen: bool,
) -> QueryResult<()> {
    let snapshot_insert = SnapshotInsert {
        server_id,
        players_online: report.players_online as i16,
        players_max: report.players_max as i16,
    };
    insert_into(schema::player_count_snapshots::table)
        .values(snapshot_insert)
        .execute(conn)
        .await?;

    diesel::sql_query(PRUNE_SQL)
        .bind::<diesel::sql_types::Integer, _>(server_id)
        .execute(conn)
        .await?;

    // De-duplicate names within this report.
    let mut seen = std::collections::HashSet::new();
    let inserts: Vec<PlayerInsert> = report
        .player_names
        .iter()
        .filter(|n| seen.insert(n.as_str()))
        .map(|n| PlayerInsert { server_id, name: n })
        .collect();

    if update_last_seen {
        insert_into(schema::players::table)
            .values(inserts)
            .on_conflict((schema::players::server_id, schema::players::name))
            .do_update()
            .set(schema::players::last_seen_at.eq(Utc::now()))
            .execute(conn)
            .await?;
    } else {
        insert_into(schema::players::table)
            .values(inserts)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
    }

    Ok(())
}

/// Deletes idempotency rows older than [`PROCESSED_RETENTION_HOURS`]. Safe to
/// call periodically; pruned ids are well past any replay window.
pub async fn prune_processed_results(db: &DatabaseWrapper) -> DbResult<usize> {
    let mut conn = db.conn().await?;
    let cutoff = Utc::now() - chrono::Duration::hours(PROCESSED_RETENTION_HOURS);
    let n = diesel::delete(
        schema::processed_results::table.filter(schema::processed_results::processed_at.lt(cutoff)),
    )
    .execute(&mut conn)
    .await?;
    Ok(n)
}

/// Fetches one keyset-paginated batch of servers a worker should re-probe this
/// cycle, honouring the spoofable/cracked filters. Rows are ordered by ascending
/// id and start strictly after `after_id`; the caller pages by passing the last
/// returned id back as `after_id` until a short batch signals the end. This
/// replaces a single load-the-whole-table query so the streaming RPC never has
/// to buffer every server at once. The pooled connection is held only for the
/// duration of this one batch query.
pub async fn fetch_update_targets_batch(
    db: &DatabaseWrapper,
    only_spoofable: bool,
    only_cracked: bool,
    after_id: Option<i32>,
    limit: i64,
) -> DbResult<Vec<ServerModelMini>> {
    let mut conn = db.conn().await?;

    let spoofable_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = if only_spoofable {
        Box::new(schema::servers::join_status.eq(JoinStatus::Spoofable))
    } else {
        Box::new(diesel::dsl::sql::<Bool>("TRUE"))
    };
    let cracked_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = if only_cracked {
        Box::new(schema::servers::is_online_mode.eq(false))
    } else {
        Box::new(diesel::dsl::sql::<Bool>("TRUE"))
    };
    let cursor_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match after_id {
        Some(id) => Box::new(schema::servers::id.gt(id)),
        None => Box::new(diesel::dsl::sql::<Bool>("TRUE")),
    };

    let servers: Vec<ServerModelMini> = schema::servers::table
        .filter(spoofable_filter)
        .filter(cracked_filter)
        .filter(cursor_filter)
        .order(schema::servers::id.asc())
        .limit(limit)
        .select(ServerModelMini::as_select())
        .load(&mut conn)
        .await?;

    Ok(servers)
}

/// Looks up a server's address by id (used to translate a frontend PingServer
/// request, which is keyed by id, into a worker ping task keyed by ip/port).
pub async fn server_addr_by_id(db: &DatabaseWrapper, id: i32) -> DbResult<Option<(String, i32)>> {
    let mut conn = db.conn().await?;
    let row = schema::servers::table
        .filter(schema::servers::id.eq(id))
        .select((schema::servers::ip, schema::servers::port))
        .first::<(String, i32)>(&mut conn)
        .await
        .optional()?;
    Ok(row)
}
