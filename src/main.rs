use crate::repository::{DbPool, QueryRepository, QueryRepositoryImpl};

async fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_connection_pool()?;
    
    let query_service = QueryServiceImpl::new(pool);
    
    Server::builder()
        .add_service(QueryServiceServer::new(query_service))
        .serve("[::1]:50051".parse()?)
        .await?;

    Ok(())
} 