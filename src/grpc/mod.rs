pub mod service;
pub mod server;
pub mod helper;
pub mod spot {
    tonic::include_proto!("spot");
}