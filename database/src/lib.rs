mod config;
mod db;
mod models;
mod persistence;
mod provider;
mod repository;
mod schema;

pub use db::{establish_connection_pool, DbConnection, DbPool};
pub use models::*;
pub use persistence::ThreadSafePersistence;
pub use repository::Repository; // Export the new thread-safe persistence

// Initialize function for the library
pub fn init(database_url: String, pool_size: u32) -> Repository {
    // Setup logging
    env_logger::init();

    // Create connection pool
    let pool = establish_connection_pool(database_url, pool_size);

    // Create repository
    let repo = Repository::new(pool);

    repo
}

// Add a new initialization function that returns the thread-safe persistence
pub fn init_thread_safe(database_url: String, pool_size: u32) -> ThreadSafePersistence {
    // Setup logging
    env_logger::init();

    // Create connection pool
    let pool = establish_connection_pool(database_url, pool_size);

    // Create thread-safe persistence
    ThreadSafePersistence::new(pool)
}
