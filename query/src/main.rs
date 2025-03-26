use spot_query::server::start_server;

#[tokio::main]
async fn main() {
    start_server("[::]:50021".to_string()).await.unwrap();
}
