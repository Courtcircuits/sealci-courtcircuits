// This file is a simple mock for testing purpose
// In order to not break the build it is commented out
// As you can see the service is missing some implementation for the moment

use agent::proto::{
    agent_server::{Agent, AgentServer},
    HealthStatus, RegisterAgentRequest, RegisterAgentResponse,
};
use tonic::{transport::Server, Response, Streaming};
#[derive(Debug, Default)]
struct RegistrationService {}

#[tonic::async_trait]
impl Agent for RegistrationService {
    async fn register_agent(
        &self,
        _request: tonic::Request<RegisterAgentRequest>,
    ) -> Result<tonic::Response<RegisterAgentResponse>, tonic::Status> {
        Ok(Response::new(RegisterAgentResponse { id: 1 }))
    }

    async fn report_health_status(
        &self,
        _request: tonic::Request<Streaming<HealthStatus>>,
    ) -> Result<tonic::Response<agent::proto::Empty>, tonic::Status> {
        Ok(Response::new(agent::proto::Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:5001".parse()?;

    let reg = RegistrationService::default();
    Server::builder()
        .add_service(AgentServer::new(reg)) // Corrected line
        .serve(addr)
        .await?;

    Ok(())
}
