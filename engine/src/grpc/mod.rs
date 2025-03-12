pub mod helper;
pub mod server;
pub mod service;
pub mod spot {
    tonic::include_proto!("spot");
}
