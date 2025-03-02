mod config;
mod db;
pub mod models;
pub mod persistence;
mod provider;
mod repository;
mod schema;

// use db::establish_connection_pool;
// use repository::Repository; // Export the new thread-safe persistence
//                             // Initialize function for the library
// pub fn init(database_url: String, pool_size: u32) -> Repository {
//     // Setup logging
//     env_logger::init();

//     // Create connection pool
//     let pool = establish_connection_pool(database_url, pool_size);

//     // Create repository
//     let repo = Repository::new(pool);

//     repo
// }
