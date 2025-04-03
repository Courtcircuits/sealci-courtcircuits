use tokio::{
    sync::mpsc::unbounded_channel,
    task::{self, JoinHandle},
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::Request;
use tracing::info;

use crate::{
    models::error::Error::{self, ConnectionError, RegistrationError},
    proto::{agent_client::AgentClient, HealthStatus, Hostname, RegisterAgentRequest},
};

use super::health_service::HealthService;

pub struct SchedulerService {
    /// This is the client that is exposed by the scheduler for the agent.
    scheduler_agent_client: AgentClient<tonic::transport::Channel>,
    health_service: HealthService,
    /// The URL that the agent will give to the scheduler.
    agent_advertise_url: String,
    port: u32,
    agent_id: Option<u32>,
}

impl SchedulerService {
    pub async fn init(
        scheduler_url: String,
        agent_host: String,
        port: u32,
        health_service: HealthService,
    ) -> Result<Self, Error> {
        info!("{}", scheduler_url.to_string());
        let scheduler_agent_client = AgentClient::connect(scheduler_url.to_string())
            .await
            .map_err(ConnectionError)?;
        let agent_advertise_url = String::from(agent_host);
        Ok(SchedulerService {
            scheduler_agent_client,
            health_service,
            agent_advertise_url,
            port,
            agent_id: None,
        })
    }

    pub async fn register(&mut self) -> Result<(), Error> {
        let host = Hostname {
            host: self.agent_advertise_url.clone(),
            port: self.port,
        };
        let health = self.health_service.get_health().await;
        let req = RegisterAgentRequest {
            health: Some(health),
            hostname: Some(host),
        };
        let request = tonic::Request::new(req);
        let res = self
            .scheduler_agent_client
            .register_agent(request)
            .await
            .map_err(RegistrationError)?
            .into_inner();
        self.agent_id = Some(res.id);
        Ok(())
    }

    pub async fn report_health(&mut self) -> Result<JoinHandle<()>, Error> {
        let (tx, rx) = unbounded_channel();
        let agent_id = self.agent_id.ok_or(Error::NotRegisteredError)?;

        let mut health_stream = self.health_service.get_health_stream();
        let health_stream_handle = task::spawn(async move {
            while let health = health_stream.next().await {
                match tx.send(HealthStatus { agent_id, health }) {
                    Err(_) => break,
                    _ => {}
                };
            }
        });
        self.scheduler_agent_client
            .report_health_status(Request::new(UnboundedReceiverStream::new(rx)))
            .await
            .map_err(Error::ReportHealthError)?;
        Ok(health_stream_handle)
    }
}
