use tonic::transport::Server;
use query_service::QueryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up database connection pool
    let pool = PgPool::connect("postgresql://username:password@localhost/dbname").await?;
    
    // Create the service
    let query_service = QueryService { pool };
    
    // Start the server
    Server::builder()
        .add_service(QueryServiceServer::new(query_service))
        .serve("[::1]:50051".parse()?)
        .await?;

    Ok(())
} 