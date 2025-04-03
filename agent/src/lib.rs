pub mod models;
pub mod server;
pub mod services;
pub mod proto {
    tonic::include_proto!("scheduler");
    tonic::include_proto!("actions");
}
pub mod config;
