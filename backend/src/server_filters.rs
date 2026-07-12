//! Optional server-property filters shared by the dashboard server list query
//! (`ApiService::list_servers`) and the worker update-target query
//! (`persistence::fetch_update_targets_batch`). Keeping the predicate
//! construction in one place means the two paths can never drift on filter
//! semantics. Mirrors the `worker.ServerFilter` proto message and the dashboard
//! `ServerListRequest` filter fields.

use crate::models::servers::JoinStatus;

/// Optional filters over the `servers` table. `None` means "no constraint".
#[derive(Debug, Default, Clone)]
pub struct ServerFilters {
    pub online: Option<bool>,
    pub licensed: Option<bool>, // is_online_mode
    pub checked: Option<bool>,
    pub crashed: Option<bool>,
    pub requires_mods: Option<bool>,
    pub has_players: Option<bool>,
    pub has_none_players: Option<bool>,
    pub join_status: Option<JoinStatus>,
    pub query: Option<String>,
}

impl From<&proto::worker::ServerFilter> for ServerFilters {
    fn from(f: &proto::worker::ServerFilter) -> Self {
        ServerFilters {
            online: f.online,
            licensed: f.licensed,
            checked: f.checked,
            crashed: f.crashed,
            requires_mods: f.requires_mods,
            has_players: f.has_players,
            has_none_players: f.has_none_players,
            join_status: f.join_status.as_deref().and_then(parse_join_status),
            query: f.query.clone(),
        }
    }
}

/// Escapes LIKE/ILIKE wildcards (`%`, `_`) and the escape char (`\`) in a
/// user-supplied search needle so it is matched literally inside `%...%`.
pub fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Parses the DB enum text of a `join_status` (case-insensitive, matching either
/// the lowercase Postgres labels or the capitalized frontend variants) into the
/// diesel enum. Returns `None` for an unknown/empty value ("no constraint").
pub fn parse_join_status(s: &str) -> Option<JoinStatus> {
    match s.trim().to_ascii_lowercase().as_str() {
        "undetermined" => Some(JoinStatus::Undetermined),
        "spoofable" => Some(JoinStatus::Spoofable),
        "whitelist" => Some(JoinStatus::Whitelist),
        "password" => Some(JoinStatus::Password),
        "modded" => Some(JoinStatus::Modded),
        "broken" => Some(JoinStatus::Broken),
        _ => None,
    }
}

/// Applies every set filter in `$filters` (a `&ServerFilters`) to a Diesel query
/// whose FROM clause includes `servers`. Works for both a bare `servers::table`
/// and a join because each predicate is boxed and its query source is inferred
/// at the call site. Returns the query with the filters chained on.
#[macro_export]
macro_rules! apply_server_filters {
    ($query:expr, $filters:expr) => {{
        use ::diesel::PgTextExpressionMethods;
        use ::diesel::dsl::{exists, not, sql};
        use ::diesel::pg::Pg;
        use ::diesel::prelude::*;
        use ::diesel::sql_types::Bool;
        use $crate::models::players::PlayerStatus;
        use $crate::schema::{players, servers};

        let f = $filters;

        let online: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> = match f.online {
            Some(v) => Box::new(servers::is_online.eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let licensed: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> = match f.licensed {
            Some(v) => Box::new(servers::is_online_mode.eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let checked: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> = match f.checked {
            Some(v) => Box::new(servers::is_checked.assume_not_null().eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let crashed: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> = match f.crashed {
            Some(v) => Box::new(servers::is_crashed.assume_not_null().eq(v)),
            None => Box::new(sql::<Bool>("TRUE")),
        };
        let requires_mods: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> =
            match f.requires_mods {
                Some(v) => Box::new(servers::requires_mods.eq(v)),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let has_players: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> =
            match f.has_players {
                Some(true) => Box::new(exists(
                    players::table.filter(players::server_id.eq(servers::id)),
                )),
                Some(false) => Box::new(not(exists(
                    players::table.filter(players::server_id.eq(servers::id)),
                ))),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let has_none_players: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> =
            match f.has_none_players {
                Some(true) => Box::new(exists(
                    players::table
                        .filter(players::server_id.eq(servers::id))
                        .filter(players::status.eq(PlayerStatus::None)),
                )),
                Some(false) => Box::new(not(exists(
                    players::table
                        .filter(players::server_id.eq(servers::id))
                        .filter(players::status.eq(PlayerStatus::None)),
                ))),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let join_status: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> =
            match f.join_status {
                Some(v) => Box::new(servers::join_status.eq(v)),
                None => Box::new(sql::<Bool>("TRUE")),
            };
        let search: Box<dyn ::diesel::BoxableExpression<_, Pg, SqlType = Bool>> = match f
            .query
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(n) => {
                let pattern = format!("%{}%", $crate::server_filters::escape_like(n));
                Box::new(
                    servers::ip
                        .ilike(pattern.clone())
                        .or(servers::version_name.ilike(pattern.clone()))
                        .or(servers::motd.ilike(pattern)),
                )
            }
            None => Box::new(sql::<Bool>("TRUE")),
        };

        $query
            .filter(online)
            .filter(licensed)
            .filter(checked)
            .filter(crashed)
            .filter(requires_mods)
            .filter(has_players)
            .filter(has_none_players)
            .filter(join_status)
            .filter(search)
    }};
}
