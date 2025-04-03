use bollard::Docker;
use clap::Parser;
use health_service::report_health;
use lazy_static::lazy_static;
use registering_service::register_agent;
use server::ActionsLauncher;
use services::container_service;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tonic::transport::Server;

use tracing::{info, error};

mod health_service;
mod models;
mod registering_service;
mod server;
mod services;
use crate::proto::action_service_server::ActionServiceServer;
mod proto {
    tonic::include_proto!("scheduler");
    tonic::include_proto!("actions");
}

lazy_static! {
    static ref AGENT_ID: Mutex<u32> = Mutex::new(0);
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The host URL of the scheduler
    #[clap(long, default_value = "http://[::1]:50051")]
    shost: String,

    /// The host URL you want the scheduler to contact the agent on
    #[clap(long, default_value = "http://[::1]")]
    ahost: String,

    /// The port of the agent to listen on
    #[clap(long, default_value = "9001")]
    port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let args: Args = Args::parse();
    info!("Connecting to scheduler at {}", args.shost);

    let (mut client, id) = match register_agent(&args.shost, &args.ahost, args.port).await {
        Ok(res) => {
            info!("Connection succeeded");
            res
        }
        Err(err) => {
            error!("Connection failed: {:?}", err);
            return Err(err);
        }
    };
    tokio::spawn(async move {
        loop {
            let _ = report_health(&mut client, id).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    info!("Agent id: {}", id);
    info!("Starting server...");
    let addr = format!("0.0.0.0:{}", args.port).parse()?;
    info!("Starting server on {}", addr);
    let docker: Arc<Docker> = Arc::new(Docker::connect_with_socket_defaults().unwrap());
    docker.ping().await?;
    let container_service = container_service::ContainerService::new(docker.clone());
    let actions = ActionsLauncher { container_service };
    let server = ActionServiceServer::new(actions);
    Server::builder().add_service(server).serve(addr).await?;
    Ok(())
}
