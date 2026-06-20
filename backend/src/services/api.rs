//! The frontend-facing gRPC service. Each method is a direct port of a former
//! REST handler, plus the new worker-management RPCs. Auth is enforced
//! per-method via `auth::require_session` (everything except `login`).

use std::{pin::Pin, sync::Arc, time::Duration};

use chrono::Utc;
use crate::{
    models::{
        player_count_snapshots::SnapshotModel,
        players::{PlayerModel, PlayerStatus as DbStatus, PlayerUpdate},
        servers::{ServerModel, ServerModelMini},
    },
    schema::{self, players, servers},
};
use diesel::{
    dsl::sql,
    pg::Pg,
    prelude::*,
    sql_types::{BigInt, Bool, Double, Nullable},
};
use diesel_async::RunQueryDsl;
use futures::Stream;
use proto::api::{
    AddAddrRequest, AddTargetListRequest, ControlWorkerRequest, DeletePlayerRequest, Empty,
    GetWorkerRequest,
    LoginRequest, LoginResponse, Player, PlayerListRequest, PlayerListResponse,
    PlayerSearchRequest, PlayerSearchResponse, PlayerSearchResult, PingServerRequest,
    OverwriteServerRequest, ServerDeleteRequest, ServerInfo, ServerInfoRequest, ServerListRequest,
    ServerListResponse, ServerSnapshot, ServerSnapshotsRequest, ServerSnapshotsResponse,
    SetWorkerNameRequest, StatsResponse, UpdatePlayerRequest, UpdateServerRequest,
    UpdateWorkerConfigRequest, VersionStat, WorkerInfo, WorkerList, api_server::Api,
};
use tokio_stream::{StreamExt, wrappers::{BroadcastStream, IntervalStream}};
use tonic::{Request, Response, Status};

use crate::{
    auth::{self, BACKEND_PASSWORD, Claims},
    chat::ChatObject,
    database::DatabaseWrapper,
    html::parse_html,
    state::AppState,
};

const SESSION_DURATION_HOURS: i64 = 24;
const DEFAULT_PORT: i32 = 25565;

pub struct ApiService {
    pub state: Arc<AppState>,
}

fn db_err<E: std::fmt::Display>(context: &str, e: E) -> Status {
    tracing::error!("{context}: {e}");
    Status::internal("database error")
}

/// Turns an affected-row count of 0 into a `NOT_FOUND` so writes against a
/// stale/deleted id surface as an error instead of a silent success (which would
/// leave the frontend's optimistic update in place).
fn require_affected(affected: usize, what: &str) -> Result<(), Status> {
    if affected == 0 {
        Err(Status::not_found(format!("{what} not found")))
    } else {
        Ok(())
    }
}

fn proto_status(s: DbStatus) -> i32 {
    match s {
        DbStatus::None => 0,
        DbStatus::Regular => 1,
        DbStatus::Admin => 2,
    }
}

fn db_status(i: i32) -> DbStatus {
    match i {
        1 => DbStatus::Regular,
        2 => DbStatus::Admin,
        _ => DbStatus::None,
    }
}

/// Case-insensitive free-text match over a server's IP, version name, and
/// plain-text MOTD. `needle` must already be lowercased.
fn server_matches(server: &ServerModel, needle: &str) -> bool {
    if server.ip.to_lowercase().contains(needle)
        || server.version_name.to_lowercase().contains(needle)
    {
        return true;
    }
    serde_json::from_value::<ChatObject>(server.description.clone())
        .map(|chat| chat.get_motd().to_lowercase().contains(needle))
        .unwrap_or(false)
}

fn server_info(server: ServerModel, snap: SnapshotModel) -> ServerInfo {
    ServerInfo {
        id: server.id,
        ip: server.ip,
        online: snap.players_online as i32,
        max: snap.players_max as i32,
        version_name: server.version_name,
        protocol: server.protocol,
        license: server.is_online_mode,
        disconnect_reason_html: server.disconnect_reason.map(parse_html),
        updated: server.updated_at.to_rfc3339(),
        description_html: parse_html(server.description),
        was_online: server.is_online,
        is_checked: server.is_checked,
        is_spoofable: server.is_spoofable,
        is_crashed: server.is_crashed,
        requires_mods: server.requires_mods,
        favicon: server.favicon,
        ping: server.ping,
    }
}

/// Loads a server's current `ServerInfo` by ip (joined with its latest snapshot).
/// Shared by the unary `GetServerInfo` and the streaming `StreamServerInfo`.
async fn load_server_info(db: &DatabaseWrapper, ip: &str) -> Result<ServerInfo, Status> {
    let mut conn = db.conn().await.map_err(|e| db_err("get conn", e))?;
    let (server, snap) = servers::table
        .inner_join(
            schema::player_count_snapshots::table
                .on(schema::player_count_snapshots::server_id.eq(servers::id)),
        )
        .filter(servers::ip.eq(ip))
        .order_by(schema::player_count_snapshots::recorded_at.desc())
        .select((ServerModel::as_select(), SnapshotModel::as_select()))
        .first::<(ServerModel, SnapshotModel)>(&mut conn)
        .await
        .map_err(|_| Status::not_found(format!("server '{ip}' not found")))?;
    Ok(server_info(server, snap))
}

/// Splits an address into `(ip, port)`.
fn parse_addr(addr: &str) -> Result<(String, i32), Status> {
    let addr = addr.trim();
    let parse_port = |s: &str| -> Result<i32, Status> {
        s.parse::<i32>()
            .map_err(|_| Status::invalid_argument(format!("Invalid port: {s}")))
    };

    let (ip, port) = if let Some(rest) = addr.strip_prefix('[') {
        let (ip, after) = rest
            .split_once(']')
            .ok_or_else(|| Status::invalid_argument(format!("Invalid address: {addr}")))?;
        let port = match after.strip_prefix(':') {
            Some(p) => parse_port(p)?,
            None if after.is_empty() => DEFAULT_PORT,
            None => return Err(Status::invalid_argument(format!("Invalid address: {addr}"))),
        };
        (ip.to_string(), port)
    } else if addr.matches(':').count() == 1 {
        let (ip, port_str) = addr.rsplit_once(':').unwrap();
        (ip.to_string(), parse_port(port_str)?)
    } else {
        (addr.to_string(), DEFAULT_PORT)
    };

    Ok((ip, port))
}

#[tonic::async_trait]
impl Api for ApiService {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let ip = request
            .metadata()
            .get("x-real-ip")
            .or_else(|| request.metadata().get("x-forwarded-for"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        if !auth::check_rate_limit(&ip) {
            tracing::warn!("Rate limit exceeded for IP: {ip}");
            return Err(Status::resource_exhausted("too many attempts"));
        }

        let password = request.into_inner().password;
        if password != *BACKEND_PASSWORD.lock().expect("password mutex poisoned") {
            tracing::warn!("Failed login attempt from IP: {ip}");
            return Err(Status::unauthenticated("invalid password"));
        }

        let now = Utc::now();
        let exp = (now + chrono::Duration::hours(SESSION_DURATION_HOURS)).timestamp() as usize;
        let token = auth::jwt_encode(&Claims {
            exp,
            iat: now.timestamp() as usize,
        })?;

        Ok(Response::new(LoginResponse { token }))
    }

    async fn me(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        Ok(Response::new(Empty {}))
    }

    async fn trigger_update(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let cfg = self
            .state
            .watchtower
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("watchtower is not configured"))?;

        // POST /v1/update kicks off a one-off watchtower run (pull + recreate).
        let url = format!("{}/v1/update", cfg.url.trim_end_matches('/'));
        let resp = reqwest::Client::new()
            .post(&url)
            .bearer_auth(&cfg.token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("watchtower request failed: {e}");
                Status::unavailable("could not reach watchtower")
            })?;

        if !resp.status().is_success() {
            let code = resp.status();
            tracing::error!("watchtower returned {code}");
            return Err(Status::internal(format!("watchtower returned {code}")));
        }

        tracing::info!("triggered watchtower stack update");
        Ok(Response::new(Empty {}))
    }

    async fn get_stats(&self, request: Request<Empty>) -> Result<Response<StatsResponse>, Status> {
        auth::require_session(&request)?;
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        let total_servers = servers::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count servers", e))?;
        let cracked_servers = servers::table
            .filter(servers::is_online_mode.eq(false))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count cracked", e))?;
        let online_servers = servers::table
            .filter(servers::is_online.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count online", e))?;
        let crashed_servers = servers::table
            .filter(servers::is_crashed.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count crashed", e))?;
        let mod_required_servers = servers::table
            .filter(servers::requires_mods.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count mods", e))?;
        let spoofable_servers = servers::table
            .filter(servers::is_spoofable.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count spoofable", e))?;
        let total_players = players::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count players", e))?;
        let admin_players = players::table
            .filter(players::status.eq(DbStatus::Admin))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("count admins", e))?;
        let avg_ping = servers::table
            .select(sql::<Nullable<Double>>("AVG(ping)::float8"))
            .get_result::<Option<f64>>(&mut conn)
            .await
            .map_err(|e| db_err("avg ping", e))?;
        let version_rows = servers::table
            .group_by(servers::version_name)
            .select((servers::version_name, diesel::dsl::count_star()))
            .order(diesel::dsl::count_star().desc())
            .limit(10)
            .load::<(String, i64)>(&mut conn)
            .await
            .map_err(|e| db_err("versions", e))?;
        let db_size_bytes = sql::<BigInt>("SELECT pg_database_size(current_database())")
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| db_err("db size", e))?;
        let favicon_size_bytes = sql::<BigInt>(
            "SELECT COALESCE(SUM(octet_length(favicon)), 0) FROM servers WHERE favicon IS NOT NULL",
        )
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| db_err("favicon size", e))?;

        Ok(Response::new(StatsResponse {
            total_servers,
            cracked_servers,
            online_servers,
            crashed_servers,
            mod_required_servers,
            spoofable_servers,
            total_players,
            admin_players,
            avg_ping,
            version_distribution: version_rows
                .into_iter()
                .map(|(version, count)| VersionStat { version, count })
                .collect(),
            db_size_mb: db_size_bytes as f64 / 1_048_576.0,
            favicon_size_mb: favicon_size_bytes as f64 / 1_048_576.0,
        }))
    }

    async fn list_servers(
        &self,
        request: Request<ServerListRequest>,
    ) -> Result<Response<ServerListResponse>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        // Free-text search. When present we filter rows in-process (the MOTD
        // lives as a JSONB chat tree that we flatten via `ChatObject::get_motd`,
        // which SQL can't do), scanning the keyset-paginated stream in batches
        // until we've gathered a full page of matches.
        let needle = body
            .query
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase);

        let mut out: Vec<ServerInfo> = Vec::new();
        let mut cursor = body.offset_id;
        let batch_size = if needle.is_some() { 256 } else { body.limit };

        'outer: loop {
        let pagination: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match cursor {
            Some(id) => Box::new(servers::id.lt(id)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let license: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.licensed {
            Some(v) => Box::new(servers::is_online_mode.eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let checked: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.checked {
            Some(v) => Box::new(servers::is_checked.assume_not_null().eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let spoofable: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.spoofable {
            Some(v) => Box::new(servers::is_spoofable.assume_not_null().eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let crashed: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.crashed {
            Some(v) => Box::new(servers::is_crashed.assume_not_null().eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let has_players: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.has_players {
            Some(true) => Box::new(diesel::dsl::exists(
                players::table.filter(players::server_id.eq(servers::id)),
            )),
            Some(false) => Box::new(diesel::dsl::not(diesel::dsl::exists(
                players::table.filter(players::server_id.eq(servers::id)),
            ))),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let has_none_players: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
            match body.has_none_players {
                Some(true) => Box::new(diesel::dsl::exists(
                    players::table
                        .filter(players::server_id.eq(servers::id))
                        .filter(players::status.eq(DbStatus::None)),
                )),
                Some(false) => Box::new(diesel::dsl::not(diesel::dsl::exists(
                    players::table
                        .filter(players::server_id.eq(servers::id))
                        .filter(players::status.eq(DbStatus::None)),
                ))),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let online: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.online {
            Some(v) => Box::new(servers::is_online.eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let requires_mods: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
            match body.requires_mods {
                Some(v) => Box::new(servers::requires_mods.eq(v)),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let batch = servers::table
            .inner_join(
                schema::player_count_snapshots::table
                    .on(schema::player_count_snapshots::server_id.eq(servers::id)),
            )
            .filter(pagination)
            .filter(license)
            .filter(checked)
            .filter(spoofable)
            .filter(crashed)
            .filter(has_players)
            .filter(has_none_players)
            .filter(online)
            .filter(requires_mods)
            .order((
                servers::id.desc(),
                schema::player_count_snapshots::recorded_at.desc(),
            ))
            .distinct_on(servers::id)
            .select((ServerModel::as_select(), SnapshotModel::as_select()))
            .limit(batch_size)
            .load::<(ServerModel, SnapshotModel)>(&mut conn)
            .await
            .map_err(|e| db_err("list servers", e))?;

        let got = batch.len() as i64;
        for (s, snap) in batch {
            cursor = Some(s.id);
            let keep = match needle.as_deref() {
                Some(n) => server_matches(&s, n),
                None => true,
            };
            if keep {
                out.push(server_info(s, snap));
                if out.len() as i64 >= body.limit {
                    break 'outer;
                }
            }
        }

        // Stop when the table is exhausted, or — with no search — after one page.
        if got < batch_size || needle.is_none() {
            break;
        }
        }

        Ok(Response::new(ServerListResponse { servers: out }))
    }

    async fn get_server_info(
        &self,
        request: Request<ServerInfoRequest>,
    ) -> Result<Response<ServerInfo>, Status> {
        auth::require_session(&request)?;
        let ip = request.into_inner().ip;
        Ok(Response::new(load_server_info(&self.state.db, &ip).await?))
    }

    type StreamServerInfoStream =
        Pin<Box<dyn Stream<Item = Result<ServerInfo, Status>> + Send + 'static>>;

    async fn stream_server_info(
        &self,
        request: Request<ServerInfoRequest>,
    ) -> Result<Response<Self::StreamServerInfoStream>, Status> {
        auth::require_session(&request)?;
        let ip = request.into_inner().ip;
        let db = self.state.db.clone();

        // Emit the current state immediately; this also 404s if the server is
        // unknown, before we commit to a long-lived stream.
        let initial = load_server_info(&db, &ip).await?;
        let server_id = initial.id;
        let rx = self.state.events.subscribe();

        // Re-emit fresh info whenever this server's row changes. On a broadcast
        // lag we reload defensively rather than risk missing our own id.
        let updates = BroadcastStream::new(rx)
            .filter_map(move |r| match r {
                Ok(id) => (id == server_id).then_some(()),
                Err(_) => Some(()),
            })
            .then(move |_| {
                let db = db.clone();
                let ip = ip.clone();
                async move { load_server_info(&db, &ip).await }
            });

        let stream = tokio_stream::once(Ok(initial)).chain(updates);
        Ok(Response::new(Box::pin(stream)))
    }

    async fn get_server_snapshots(
        &self,
        request: Request<ServerSnapshotsRequest>,
    ) -> Result<Response<ServerSnapshotsResponse>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        let results: Vec<SnapshotModel> = schema::player_count_snapshots::table
            .filter(schema::player_count_snapshots::server_id.eq(body.server_id))
            .order(schema::player_count_snapshots::recorded_at.desc())
            .limit(body.limit)
            .load(&mut conn)
            .await
            .map_err(|e| db_err("load snapshots", e))?;

        Ok(Response::new(ServerSnapshotsResponse {
            snapshots: results
                .into_iter()
                .map(|s| ServerSnapshot {
                    server_id: s.server_id,
                    players_online: s.players_online as i32,
                    players_max: s.players_max as i32,
                    recorded_at: s.recorded_at.to_rfc3339(),
                })
                .collect(),
        }))
    }

    async fn update_server(
        &self,
        request: Request<UpdateServerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        #[derive(AsChangeset)]
        #[diesel(table_name = servers)]
        struct Options {
            is_checked: Option<bool>,
            is_spoofable: Option<bool>,
            is_crashed: Option<bool>,
        }

        let affected = diesel::update(servers::table)
            .filter(servers::ip.eq(&body.server_ip))
            .set(Options {
                is_checked: body.is_checked,
                is_spoofable: body.is_spoofable,
                is_crashed: body.is_crashed,
            })
            .execute(&mut conn)
            .await
            .map_err(|e| db_err("update server", e))?;
        require_affected(affected, "server")?;

        Ok(Response::new(Empty {}))
    }

    async fn overwrite_server(
        &self,
        request: Request<OverwriteServerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        #[derive(AsChangeset, Default)]
        #[diesel(table_name = servers)]
        struct Overwrite {
            port: Option<i32>,
            version_name: Option<String>,
            protocol: Option<i32>,
            is_online_mode: Option<bool>,
            requires_mods: Option<bool>,
            is_online: Option<bool>,
            ping: Option<i64>,
            favicon: Option<String>,
            is_checked: Option<bool>,
            is_spoofable: Option<bool>,
            is_crashed: Option<bool>,
        }

        let affected = diesel::update(servers::table)
            .filter(servers::id.eq(body.server_id))
            .set(Overwrite {
                port: body.port,
                version_name: body.version_name,
                protocol: body.protocol,
                is_online_mode: body.is_online_mode,
                requires_mods: body.requires_mods,
                is_online: body.is_online,
                ping: body.ping,
                favicon: body.favicon,
                is_checked: body.is_checked,
                is_spoofable: body.is_spoofable,
                is_crashed: body.is_crashed,
            })
            .execute(&mut conn)
            .await
            .map_err(|e| db_err("overwrite server", e))?;
        require_affected(affected, "server")?;

        Ok(Response::new(Empty {}))
    }

    async fn delete_server(
        &self,
        request: Request<ServerDeleteRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let id = request.into_inner().id;
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;
        let affected = diesel::delete(servers::table.filter(servers::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| db_err("delete server", e))?;
        require_affected(affected, "server")?;
        Ok(Response::new(Empty {}))
    }

    async fn ping_server(
        &self,
        request: Request<PingServerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let addr = crate::persistence::server_addr_by_id(&self.state.db, body.server_id)
            .await
            .map_err(|e| db_err("resolve server", e))?
            .ok_or_else(|| Status::not_found("server not found"))?;
        self.state
            .registry
            .dispatch_ping(&body.worker_id, addr.0, addr.1, body.with_connection)
            .await?;
        Ok(Response::new(Empty {}))
    }

    async fn add_target(
        &self,
        request: Request<AddAddrRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let (ip, port) = parse_addr(&body.addr)?;
        self.state
            .registry
            .dispatch_scan_to(&body.worker_id, ip, port)
            .await?;
        Ok(Response::new(Empty {}))
    }

    async fn add_target_list(
        &self,
        request: Request<AddTargetListRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        // Validate every address up front so a malformed entry rejects the whole
        // batch before any target is dispatched.
        let parsed = body
            .targets
            .iter()
            .map(|t| parse_addr(&t.addr))
            .collect::<Result<Vec<_>, _>>()?;
        // Fail-fast: the operator picks the worker; if it is unknown or offline the
        // whole import errors rather than silently dropping work.
        for (ip, port) in parsed {
            self.state
                .registry
                .dispatch_scan_to(&body.worker_id, ip, port)
                .await?;
        }
        Ok(Response::new(Empty {}))
    }

    async fn list_players(
        &self,
        request: Request<PlayerListRequest>,
    ) -> Result<Response<PlayerListResponse>, Status> {
        auth::require_session(&request)?;
        let server_id = request.into_inner().server_id;
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        let result = players::table
            .filter(players::server_id.eq(server_id))
            .select(PlayerModel::as_select())
            .order(players::last_seen_at.desc())
            .load::<PlayerModel>(&mut conn)
            .await
            .map_err(|e| db_err("list players", e))?;

        Ok(Response::new(PlayerListResponse {
            players: result
                .into_iter()
                .map(|p| Player {
                    id: p.id,
                    server_id: p.server_id,
                    name: p.name,
                    status: proto_status(p.status),
                    last_seen_at: p.last_seen_at.to_rfc3339(),
                })
                .collect(),
        }))
    }

    async fn search_players(
        &self,
        request: Request<PlayerSearchRequest>,
    ) -> Result<Response<PlayerSearchResponse>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;

        let pagination: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.offset_id {
            Some(id) => Box::new(players::id.lt(id)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let name_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> =
            match body.name_contains {
                Some(ref s) if !s.is_empty() => Box::new(players::name.ilike(format!("%{}%", s))),
                _ => Box::new(sql::<Bool>("TRUE")),
            };
        let status_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.status {
            Some(s) => Box::new(players::status.eq(db_status(s))),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let licensed_filter: Box<dyn BoxableExpression<_, Pg, SqlType = Bool>> = match body.licensed
        {
            Some(v) => Box::new(servers::is_online_mode.eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };

        let results = players::table
            .inner_join(servers::table.on(servers::id.eq(players::server_id)))
            .filter(pagination)
            .filter(name_filter)
            .filter(status_filter)
            .filter(licensed_filter)
            .order(players::id.desc())
            .select((PlayerModel::as_select(), ServerModelMini::as_select()))
            .limit(body.limit)
            .load::<(PlayerModel, ServerModelMini)>(&mut conn)
            .await
            .map_err(|e| db_err("search players", e))?;

        Ok(Response::new(PlayerSearchResponse {
            players: results
                .into_iter()
                .map(|(player, server)| PlayerSearchResult {
                    id: player.id,
                    server_id: player.server_id,
                    server_ip: server.ip,
                    name: player.name,
                    status: proto_status(player.status),
                    last_seen_at: player.last_seen_at.to_rfc3339(),
                })
                .collect(),
        }))
    }

    async fn update_player(
        &self,
        request: Request<UpdatePlayerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;
        let status = db_status(body.status);
        let affected = diesel::update(players::table)
            .filter(players::id.eq(body.id))
            .set(&PlayerUpdate { status: &status })
            .execute(&mut conn)
            .await
            .map_err(|e| db_err("update player", e))?;
        require_affected(affected, "player")?;
        Ok(Response::new(Empty {}))
    }

    async fn delete_player(
        &self,
        request: Request<DeletePlayerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let id = request.into_inner().id;
        let mut conn = self
            .state
            .db
            .conn()
            .await
            .map_err(|e| db_err("get conn", e))?;
        let affected = diesel::delete(players::table.filter(players::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| db_err("delete player", e))?;
        require_affected(affected, "player")?;
        Ok(Response::new(Empty {}))
    }

    // ----- Worker management -----

    async fn list_workers(&self, request: Request<Empty>) -> Result<Response<WorkerList>, Status> {
        auth::require_session(&request)?;
        Ok(Response::new(self.state.registry.list().await))
    }

    type StreamWorkersStream =
        Pin<Box<dyn Stream<Item = Result<WorkerList, Status>> + Send + 'static>>;

    async fn stream_workers(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::StreamWorkersStream>, Status> {
        auth::require_session(&request)?;
        let registry = self.state.registry.clone();
        let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(2))).then(
            move |_| {
                let registry = registry.clone();
                async move { Ok(registry.list().await) }
            },
        );
        Ok(Response::new(Box::pin(stream)))
    }

    async fn get_worker(
        &self,
        request: Request<GetWorkerRequest>,
    ) -> Result<Response<WorkerInfo>, Status> {
        auth::require_session(&request)?;
        let id = request.into_inner().worker_id;
        Ok(Response::new(self.state.registry.get(&id).await?))
    }

    async fn update_worker_config(
        &self,
        request: Request<UpdateWorkerConfigRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        let config = body
            .config
            .ok_or_else(|| Status::invalid_argument("missing config"))?;
        self.state.registry.set_config(&body.worker_id, config).await?;
        Ok(Response::new(Empty {}))
    }

    async fn set_worker_name(
        &self,
        request: Request<SetWorkerNameRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        // Treat blank input as clearing the name.
        let name = body
            .name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self.state.registry.set_name(&body.worker_id, name).await?;
        Ok(Response::new(Empty {}))
    }

    async fn control_worker(
        &self,
        request: Request<ControlWorkerRequest>,
    ) -> Result<Response<Empty>, Status> {
        auth::require_session(&request)?;
        let body = request.into_inner();
        self.state
            .registry
            .send_control(&body.worker_id, body.control)
            .await?;
        Ok(Response::new(Empty {}))
    }
}
