use std::{
    net::{AddrParseError, SocketAddr},
    sync::Arc,
};

use bollard::Docker;
use clap::Parser;
use tokio::task;
use tonic::transport::Server;
use tracing::info;

use crate::{
    config::Config,
    models::error::Error,
    proto::action_service_server::ActionServiceServer,
    server::ActionsLauncher,
    services::{
        action_service::ActionService, health_service::HealthService,
        scheduler_service::SchedulerService,
    },
};

#[derive(Clone)]
pub struct App {
    config: Config,
    scheduler_service: SchedulerService,
    action_service_grpc: ActionServiceServer<ActionsLauncher>,
}

impl App {
    pub async fn init() -> Result<Self, Error> {
        let config = Config::parse();
        let health_service = HealthService::new();

        let docker = Arc::new(Docker::connect_with_socket_defaults().unwrap());
        docker.ping().await.map_err(Error::DockerConnectionError)?;
        let action_service = ActionService::new(docker);
        let actions = ActionsLauncher { action_service };
        let action_service_grpc = ActionServiceServer::new(actions);
        let mut scheduler_service = SchedulerService::init(
            config.shost.clone(),
            config.ahost.clone(),
            config.port.clone(),
            health_service,
        )
        .await?;
        scheduler_service.register().await?;
        Ok(Self {
            action_service_grpc,
            config,
            scheduler_service,
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port)
            .parse()
            .map_err(|e: AddrParseError| Error::Error(e.to_string()))?;
        info!("Starting server on {}", addr);
        let server = Server::builder()
            .add_service(self.action_service_grpc.clone())
            .serve(addr);
        let mut service = self.clone();
        let health_report = task::spawn(async move {
            let _ = service.scheduler_service.report_health().await;
        });
        tokio::select! {
            serve_res = server => {
                serve_res
            .map_err(Error::ServeError)?;
            }
            health_report = health_report => {
                let _ = health_report;
            }
        };

        Ok(())
    }
}
