pub mod health_service;
pub mod models;
pub mod registering_service;
pub mod server;
pub mod services;
pub mod proto {
    tonic::include_proto!("scheduler");
    tonic::include_proto!("actions");
}