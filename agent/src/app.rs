use std::{
    net::{AddrParseError, SocketAddr},
    sync::Arc,
};

use actix_web::HttpServer;
use bollard::Docker;
use clap::Parser;
use tokio::{sync::Mutex, task};
use tonic::transport::Server;
use tracing::info;

use crate::{
    brokers::state_broker::StateBroker,
    config::Config,
    controllers::http::ActionController,
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
    http_action_controller: ActionController,
    scheduler_service: SchedulerService,
    pub action_service: Arc<Mutex<ActionService>>,
    action_service_grpc: ActionServiceServer<ActionsLauncher>,
}

impl App {
    pub async fn init() -> Result<Self, Error> {
        let config = Config::parse();
        let health_service = HealthService::new();

        let docker = Arc::new(Docker::connect_with_socket_defaults().unwrap());
        docker.ping().await.map_err(Error::DockerConnectionError)?;

        let state_broker = Arc::new(StateBroker::new());
        let action_service = Arc::new(Mutex::new(ActionService::new(docker, state_broker.clone())));
        let http_action_controller = ActionController::new(action_service.clone());
        let actions = ActionsLauncher {
            action_service: action_service.clone(),
        };
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
            action_service,
            http_action_controller,
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port)
            .parse()
            .map_err(|e: AddrParseError| Error::Error(e.to_string()))?;
        info!("Starting grpc server on {}", addr);
        let server = Server::builder()
            .add_service(self.action_service_grpc.clone())
            .serve(addr);
        let mut service = self.clone();
        let health_report = task::spawn(async move {
            let _ = service.scheduler_service.report_health().await;
        });

        let http_action_controller = self.http_action_controller.clone();

        info!("Starting web server on {}", 8080);

        let http_server = HttpServer::new(move || {
            actix_web::App::new().configure(|cfg| http_action_controller.clone().register(cfg))
        })
        .bind(("0.0.0.0", 8080))
        .map_err(|err| Error::Error(err.to_string()))?
        .run();
        tokio::select! {
            serve_res = server => {
                serve_res
            .map_err(Error::ServeError)?;
            }
            http_server = http_server => {
                let _ = http_server;
            }
            health_report = health_report => {
                let _ = health_report;
            }
        };

        Ok(())
    }
}
