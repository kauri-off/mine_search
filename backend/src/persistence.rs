//! Database writes for results streamed in from workers. This is the logic that
//! used to live in the worker (`handle_valid_ip` / `update_server`) — now the
//! backend is the sole writer and workers only report structured outcomes.

use chrono::Utc;
use crate::{
    models::{
        player_count_snapshots::SnapshotInsert,
        players::PlayerInsert,
        servers::{ServerExtraUpdate, ServerInsert, ServerModel, ServerModelMini, ServerUpdate},
    },
    schema,
};
use diesel::{dsl::insert_into, pg::Pg, prelude::*, sql_types::Bool};
use diesel_async::RunQueryDsl;
use proto::worker::{ServerReport, UpdateTarget};

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

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
}

/// Mirrors the old `handle_valid_ip`: upsert-by-ip, conflict only bumps
/// updated_at/is_online/favicon (preserving the original quirk that rediscovery
/// does not overwrite version/description).
pub async fn persist_discovered(db: &DatabaseWrapper, report: ServerReport) -> DbResult<Option<i32>> {
    let description = parse_json(&report.description_json);
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
        is_online_mode,
        disconnect_reason,
        requires_mods: report.requires_mods,
        favicon,
        ping: report.ping,
    };

    let mut conn = db.pool.get().await?;

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
        .get_result(&mut conn)
        .await?;

    write_snapshot_and_players(db, server.id, &report, false).await?;
    Ok(Some(server.id))
}

/// Mirrors the old `update_server` (reachable path): full field update by ip.
pub async fn persist_updated(db: &DatabaseWrapper, report: ServerReport) -> DbResult<Option<i32>> {
    let mut conn = db.pool.get().await?;

    let Some(server_id) = schema::servers::table
        .filter(schema::servers::ip.eq(&report.ip))
        .select(schema::servers::id)
        .first::<i32>(&mut conn)
        .await
        .optional()?
    else {
        // Server vanished between scheduling and reporting — nothing to update.
        return Ok(None);
    };

    let description = parse_json(&report.description_json);
    let favicon = report.favicon.as_deref();

    let server_change = ServerUpdate {
        version_name: &report.version_name,
        protocol: report.protocol,
        description: &description,
        updated_at: Utc::now(),
        is_online: true,
        requires_mods: report.requires_mods,
        favicon,
        ping: report.ping,
    };

    diesel::update(schema::servers::table)
        .filter(schema::servers::id.eq(server_id))
        .set(server_change)
        .execute(&mut conn)
        .await?;

    if let Some(extra) = &report.extra {
        let server_extra_change = ServerExtraUpdate {
            is_online_mode: extra.is_online_mode,
            disconnect_reason: extra.disconnect_reason_json.as_deref().map(parse_json),
        };
        diesel::update(schema::servers::table)
            .filter(schema::servers::id.eq(server_id))
            .set(server_extra_change)
            .execute(&mut conn)
            .await?;
    }

    drop(conn);
    write_snapshot_and_players(db, server_id, &report, true).await?;
    Ok(Some(server_id))
}

/// Marks a server offline after a failed re-probe (old `update_server` error path).
pub async fn persist_offline(db: &DatabaseWrapper, ip: &str) -> DbResult<Option<i32>> {
    let mut conn = db.pool.get().await?;
    let id = diesel::update(schema::servers::table)
        .filter(schema::servers::ip.eq(ip))
        .set(schema::servers::is_online.eq(false))
        .returning(schema::servers::id)
        .get_result::<i32>(&mut conn)
        .await
        .optional()?;
    Ok(id)
}

/// Inserts a player-count snapshot, prunes old ones, and records players. When
/// `update_last_seen` is set, existing players have their `last_seen_at` bumped
/// (update cycle); otherwise duplicates are ignored (discovery).
async fn write_snapshot_and_players(
    db: &DatabaseWrapper,
    server_id: i32,
    report: &ServerReport,
    update_last_seen: bool,
) -> DbResult<()> {
    let mut conn = db.pool.get().await?;

    let snapshot_insert = SnapshotInsert {
        server_id,
        players_online: report.players_online as i16,
        players_max: report.players_max as i16,
    };
    insert_into(schema::player_count_snapshots::table)
        .values(snapshot_insert)
        .execute(&mut conn)
        .await?;

    diesel::sql_query(PRUNE_SQL)
        .bind::<diesel::sql_types::Integer, _>(server_id)
        .execute(&mut conn)
        .await?;

    // De-duplicate names within this report.
    let mut seen = std::collections::HashSet::new();
    let inserts: Vec<PlayerInsert> = report
        .player_names
        .iter()
        .filter(|n| seen.insert(n.as_str()))
        .map(|n| PlayerInsert {
            server_id,
            name: n,
        })
        .collect();

    if update_last_seen {
        insert_into(schema::players::table)
            .values(inserts)
            .on_conflict((schema::players::server_id, schema::players::name))
            .do_update()
            .set(schema::players::last_seen_at.eq(Utc::now()))
            .execute(&mut conn)
            .await?;
    } else {
        insert_into(schema::players::table)
            .values(inserts)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

/// Returns the servers a worker should re-probe this cycle (old `updater`
/// selection), honouring the spoofable/cracked filters and stamping each target
/// with the worker's `with_connection` preference.
pub async fn fetch_update_targets(
    db: &DatabaseWrapper,
    only_spoofable: bool,
    only_cracked: bool,
    with_connection: bool,
) -> DbResult<Vec<UpdateTarget>> {
    let mut conn = db.pool.get().await?;

    let spoofable_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = if only_spoofable {
        Box::new(schema::servers::is_spoofable.assume_not_null().eq(true))
    } else {
        Box::new(diesel::dsl::sql::<Bool>("TRUE"))
    };
    let cracked_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = if only_cracked {
        Box::new(schema::servers::is_online_mode.eq(false))
    } else {
        Box::new(diesel::dsl::sql::<Bool>("TRUE"))
    };

    let servers: Vec<ServerModelMini> = schema::servers::table
        .filter(spoofable_filter)
        .filter(cracked_filter)
        .select(ServerModelMini::as_select())
        .load(&mut conn)
        .await?;

    Ok(servers
        .into_iter()
        .map(|s| UpdateTarget {
            ip: s.ip,
            port: s.port,
            with_connection,
        })
        .collect())
}

/// Looks up a server's address by id (used to translate a frontend PingServer
/// request, which is keyed by id, into a worker ping task keyed by ip/port).
pub async fn server_addr_by_id(db: &DatabaseWrapper, id: i32) -> DbResult<Option<(String, i32)>> {
    let mut conn = db.pool.get().await?;
    let row = schema::servers::table
        .filter(schema::servers::id.eq(id))
        .select((schema::servers::ip, schema::servers::port))
        .first::<(String, i32)>(&mut conn)
        .await
        .optional()?;
    Ok(row)
}
