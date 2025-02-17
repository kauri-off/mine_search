use std::env;

use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};

pub struct DatabaseWrapper {
    pub pool: Pool<AsyncPgConnection>,
}

impl DatabaseWrapper {
    pub fn establish() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().unwrap();

        Self { pool }
    }
}
