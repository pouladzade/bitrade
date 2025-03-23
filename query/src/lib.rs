pub mod adapter;
pub mod server;
pub mod service;
pub mod spot_query {
    tonic::include_proto!("spot_query");
}
