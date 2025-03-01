// db.rs
// Database connection and pooling setup

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
// Type alias for a pooled PostgreSQL connection
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Create a new database connection pool
pub fn establish_connection_pool(database_url: String, pool_size: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    diesel::r2d2::Pool::builder()
        .max_size(pool_size) // Maximum number of connections in the pool
        .build(manager)
        .expect("Failed to create connection pool")
}

/// Get a connection from the pool
pub fn get_connection(pool: &DbPool) -> DbConnection {
    pool.get()
        .expect("Failed to get a connection from the pool")
}
