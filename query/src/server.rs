use database::establish_connection_pool;
use database::repository::Repository;

use crate::service::SpotQueryServiceImp;
use crate::spot_query::spot_query_service_server::SpotQueryServiceServer;
use log::info;
use std::env;
use tonic::transport::Server;

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let adr = address.parse().unwrap();
    info!("P2P Server listening on {}", address);

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:mysecretpassword@postgres:5432/postgres".to_string()
    });
    let pool_size = 10;
    let pool = establish_connection_pool(database_url, pool_size);
    let repository = Repository::new(pool);
    if let Err(e) = Server::builder()
        .add_service(SpotQueryServiceServer::new(SpotQueryServiceImp::new(
            repository,
        )))
        .serve(adr)
        .await
    {
        println!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
