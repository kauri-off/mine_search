use std::env;

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};

pub struct DatabaseWrapper {
    pub pool: Pool<AsyncPgConnection>,
}

impl DatabaseWrapper {
    pub fn establish() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().expect("Failed to build DB connection pool");

        Self { pool }
    }
}
