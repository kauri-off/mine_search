use std::time::Duration;

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
};

/// Upper bound on pooled connections. Caps concurrent DB work so a flood of
/// worker results or API calls can't open unbounded connections.
const MAX_POOL_SIZE: usize = 16;

/// Hard ceiling on acquiring a connection. Without this, `pool.get()` rides the
/// OS TCP-connect timeout when Postgres is unreachable, blocking callers for
/// many seconds; bounding it lets writes fail fast (and be retried/replayed).
const GET_TIMEOUT: Duration = Duration::from_secs(5);

pub type PooledConn = Object<AsyncPgConnection>;

pub struct DatabaseWrapper {
    pub pool: Pool<AsyncPgConnection>,
}

impl DatabaseWrapper {
    pub fn establish(database_url: &str) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config)
            .max_size(MAX_POOL_SIZE)
            .build()
            .expect("Failed to build DB connection pool");

        Self { pool }
    }

    /// Fetches a pooled connection, bounded by [`GET_TIMEOUT`]. Prefer this over
    /// `pool.get()` directly so a dead/unreachable database fails fast.
    pub async fn conn(&self) -> Result<PooledConn, Box<dyn std::error::Error + Send + Sync>> {
        match tokio::time::timeout(GET_TIMEOUT, self.pool.get()).await {
            Ok(Ok(conn)) => Ok(conn),
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => Err("timed out acquiring database connection".into()),
        }
    }
}
