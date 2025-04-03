use crate::proto::{
    action_service_server::ActionService as ActionServiceGrpc, ActionRequest, ActionResponseStream,
};
use crate::services::action_service::ActionService;
use futures_util::Stream;
use std::pin::Pin;
use tokio::sync::mpsc::{self};
use tokio::task;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{async_trait, Request, Response, Status};

pub struct ActionsLauncher {
    pub action_service: ActionService,
}

#[async_trait]
impl ActionServiceGrpc for ActionsLauncher {
    type ExecutionActionStream =
        Pin<Box<dyn Stream<Item = Result<ActionResponseStream, Status>> + Send>>;

    async fn execution_action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<Self::ExecutionActionStream>, Status> {
        let (log_input, log_ouput) =
            mpsc::unbounded_channel::<Result<ActionResponseStream, Status>>();
        let request_body = request.into_inner();
        let context = request_body
            .context
            .ok_or_else(|| Status::invalid_argument("Context is missing"))?;
        let container_image = context
            .container_image
            .ok_or_else(|| Status::invalid_argument("Container image is missing"))?;

        let action = self
            .action_service
            .create(
                container_image,
                request_body.commands,
                log_input,
                request_body.repo_url,
                request_body.action_id,
            )
            .await
            .map_err(|_| Status::failed_precondition("Failed to create action"))?;
        task::spawn(async move { action.execute().await });
        Ok(Response::new(Box::pin(UnboundedReceiverStream::new(
            log_ouput,
        ))))
    }
}
