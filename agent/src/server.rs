use crate::models::action::Action;
use crate::proto::{action_service_server::ActionService, ActionRequest, ActionResponseStream};
use crate::services::container_service::ContainerService;
use futures_util::Stream;
use std::pin::Pin;
use tokio::sync::mpsc::{self};
use tokio::task;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use tracing::debug;

pub struct ActionsLauncher {
    pub container_service: ContainerService,
}

#[async_trait]
impl ActionService for ActionsLauncher {
    type ExecutionActionStream =
        Pin<Box<dyn Stream<Item = Result<ActionResponseStream, Status>> + Send>>;

    async fn execution_action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<Self::ExecutionActionStream>, Status> {
        let (log_input, log_ouput) =
            mpsc::unbounded_channel::<Result<ActionResponseStream, Status>>();
        let request_body = request.into_inner();
        let context = match request_body.context {
            Some(context) => context,
            None => return Err(Status::invalid_argument("Context is missing")),
        };
        let container_image = match context.container_image {
            Some(container_image) => container_image,
            None => return Err(Status::invalid_argument("Container image is missing")),
        };
        debug!(
            "Creating container with image {} for action {}",
            request_body.repo_url, request_body.action_id
        );
        let container = match self
            .container_service
            .create_container(container_image)
            .await
        {
            Ok(container) => container,
            Err(_) => return Err(Status::aborted(format!("Could not create container"))),
        };
        let action = Action::new(
            request_body.action_id,
            container,
            request_body.commands,
            log_input,
            request_body.repo_url,
        );
        debug!(
            "Cloning repository {} for action {}",
            action.repository_url, action.id
        );
        match action.setup_repository().await {
            Ok(_) => (),
            Err(_) => return Err(Status::aborted(format!("Could not setup repository"))),
        };
        debug!("Starting the execution of steps for action {}", action.id);
        task::spawn(async move {
            match action.execute().await {
                Ok(_) => Ok(()),
                Err(_) => return Err(Status::aborted(format!("Could not execute action"))),
            }
        });
        let stream = UnboundedReceiverStream::new(log_ouput);
        Ok(Response::new(
            Box::pin(stream) as Self::ExecutionActionStream
        ))
    }
}
