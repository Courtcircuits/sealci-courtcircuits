use agent::config::Config;
use agent::models::error::Error;
use agent::proto::action_service_server::ActionServiceServer;
use agent::server;
use agent::services::action_service::ActionService;
use agent::services::health_service::HealthService;
use agent::services::scheduler_service::SchedulerService;
use bollard::Docker;
use clap::Parser;
use server::ActionsLauncher;
use std::net::AddrParseError;
use std::sync::Arc;
use tonic::transport::Server;

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let config = Config::parse();
    let health_service = HealthService::new();
    let mut scheduler_service =
        SchedulerService::init(config.shost, config.ahost, config.port, health_service).await?;
    scheduler_service.register().await?;
    let health_stream_handle = scheduler_service.report_health().await?;
    let addr = format!("0.0.0.0:{}", config.port)
        .parse()
        .map_err(|e: AddrParseError| Error::Error(e.to_string()))?;
    info!("Starting server on {}", addr);
    let docker: Arc<Docker> = Arc::new(Docker::connect_with_socket_defaults().unwrap());
    docker.ping().await.map_err(Error::DockerConnectionError)?;
    let action_service = ActionService::new(docker);
    let actions = ActionsLauncher { action_service };
    let server = ActionServiceServer::new(actions);
    Server::builder()
        .add_service(server)
        .serve(addr)
        .await
        .map_err(Error::ServeError)?;
    let _ = health_stream_handle.await;
    Ok(())
}
