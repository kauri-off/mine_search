//! `diesel` feature (default OFF): the legacy mode where the worker talks
//! directly to PostgreSQL. The Sink writes scan results to the DB and the
//! TargetSource reads the server list from it. There is no backend↔worker
//! channel in this mode, so on-demand ping/scan from the API is unavailable.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use db_schema::{
    models::{
        player_count_snapshots::SnapshotInsert,
        players::PlayerInsert,
        servers::{ServerExtraUpdate, ServerInsert, ServerModel, ServerModelMini, ServerUpdate},
    },
    schema,
};
use diesel::{dsl::insert_into, pg::Pg, prelude::*, sql_types::Bool};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::error;

use crate::{
    report::ScanReport,
    sink::{Sink, TargetSource, UpdateTarget},
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../db_schema/migrations");

const PRUNE_SQL: &str = "DELETE FROM player_count_snapshots \
     WHERE server_id = $1 \
     AND recorded_at < ( \
         SELECT recorded_at FROM player_count_snapshots \
         WHERE server_id = $1 \
         ORDER BY recorded_at DESC \
         LIMIT 1 OFFSET 99 \
     )";

pub struct Database {
    pub pool: Pool<AsyncPgConnection>,
}

impl Database {
    pub fn establish(url: &str) -> Arc<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        let pool = Pool::builder(config)
            .build()
            .expect("Failed to build DB connection pool");
        Arc::new(Self { pool })
    }
}

pub fn run_migrations(url: &str) {
    use diesel::{Connection, PgConnection};
    let mut conn = PgConnection::establish(url).expect("Failed to connect for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

pub struct DieselSink {
    pub db: Arc<Database>,
}

impl DieselSink {
    async fn snapshot_and_players(
        &self,
        server_id: i32,
        report: &ScanReport,
        update_last_seen: bool,
    ) -> anyhow::Result<()> {
        let mut conn = self.db.pool.get().await?;

        insert_into(schema::player_count_snapshots::table)
            .values(SnapshotInsert {
                server_id,
                players_online: report.players_online as i16,
                players_max: report.players_max as i16,
            })
            .execute(&mut conn)
            .await?;

        diesel::sql_query(PRUNE_SQL)
            .bind::<diesel::sql_types::Integer, _>(server_id)
            .execute(&mut conn)
            .await?;

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

    async fn do_discovered(&self, report: ScanReport) -> anyhow::Result<()> {
        let (is_online_mode, disconnect_reason) = match &report.extra {
            Some(e) => (e.is_online_mode, e.disconnect_reason.clone()),
            None => (false, None),
        };
        let favicon = report.favicon.as_deref();

        let mut conn = self.db.pool.get().await?;
        let server: ServerModel = insert_into(schema::servers::table)
            .values(&ServerInsert {
                ip: &report.ip,
                port: report.port,
                version_name: &report.version_name,
                protocol: report.protocol,
                description: &report.description,
                is_online_mode,
                disconnect_reason,
                requires_mods: report.requires_mods,
                favicon,
                ping: report.ping,
            })
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
        drop(conn);

        self.snapshot_and_players(server.id, &report, false).await
    }

    async fn do_updated(&self, report: ScanReport) -> anyhow::Result<()> {
        let mut conn = self.db.pool.get().await?;
        let Some(server_id) = schema::servers::table
            .filter(schema::servers::ip.eq(&report.ip))
            .select(schema::servers::id)
            .first::<i32>(&mut conn)
            .await
            .optional()?
        else {
            return Ok(());
        };

        let favicon = report.favicon.as_deref();
        diesel::update(schema::servers::table)
            .filter(schema::servers::id.eq(server_id))
            .set(ServerUpdate {
                version_name: &report.version_name,
                protocol: report.protocol,
                description: &report.description,
                updated_at: Utc::now(),
                is_online: true,
                requires_mods: report.requires_mods,
                favicon,
                ping: report.ping,
            })
            .execute(&mut conn)
            .await?;

        if let Some(extra) = &report.extra {
            diesel::update(schema::servers::table)
                .filter(schema::servers::id.eq(server_id))
                .set(ServerExtraUpdate {
                    is_online_mode: extra.is_online_mode,
                    disconnect_reason: extra.disconnect_reason.clone(),
                })
                .execute(&mut conn)
                .await?;
        }
        drop(conn);

        self.snapshot_and_players(server_id, &report, true).await
    }
}

#[async_trait]
impl Sink for DieselSink {
    async fn discovered(&self, report: ScanReport) {
        if let Err(e) = self.do_discovered(report).await {
            error!("diesel discovered failed: {e}");
        }
    }
    async fn updated(&self, report: ScanReport) {
        if let Err(e) = self.do_updated(report).await {
            error!("diesel updated failed: {e}");
        }
    }
    async fn offline(&self, ip: &str) {
        let res = async {
            let mut conn = self.db.pool.get().await?;
            diesel::update(schema::servers::table)
                .filter(schema::servers::ip.eq(ip))
                .set(schema::servers::is_online.eq(false))
                .execute(&mut conn)
                .await?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        if let Err(e) = res {
            error!("diesel offline failed: {e}");
        }
    }
}

pub struct DieselTargetSource {
    pub db: Arc<Database>,
}

#[async_trait]
impl TargetSource for DieselTargetSource {
    async fn update_targets(
        &self,
        only_spoofable: bool,
        only_cracked: bool,
        with_connection: bool,
    ) -> anyhow::Result<Vec<UpdateTarget>> {
        let mut conn = self.db.pool.get().await?;

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
                port: s.port as u16,
                with_connection,
            })
            .collect())
    }
}
