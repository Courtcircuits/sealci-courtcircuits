use crate::proto::{
    action_service_server::ActionService as ActionServiceGrpc, ActionRequest, ActionResponseStream,
};
use crate::services::action_service::ActionService;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{oneshot, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use tracing::info;

pub struct ActionsLauncher {
    pub action_service: Arc<Mutex<ActionService>>,
}

#[async_trait]
impl ActionServiceGrpc for ActionsLauncher {
    type ExecutionActionStream =
        Pin<Box<dyn Stream<Item = Result<ActionResponseStream, Status>> + Send>>;

    async fn execution_action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<Self::ExecutionActionStream>, Status> {
        // Create two channels:
        // 1. For normal log messages
        let (log_tx, log_rx) = unbounded_channel::<Result<ActionResponseStream, Status>>();

        // 2. For signaling completion
        let (done_tx, done_rx) = oneshot::channel::<()>();

        let request_body = request.into_inner();
        let context = request_body
            .context
            .ok_or(Status::invalid_argument("Context is missing"))?;
        let container_image = context
            .container_image
            .ok_or(Status::invalid_argument("Container image is missing"))?;

        let mut action = self
            .action_service.lock()
            .await
            .create(
                container_image,
                request_body.commands,
                log_tx.clone(),
                request_body.repo_url,
                request_body.action_id,
            )
            .await
            .map_err(|_| Status::failed_precondition("Failed to create action"))?;

        // Spawn a task to execute the action and signal completion
        tokio::spawn(async move {
            let _ = action.execute().await;
            info!("Action executed");

            // Signal completion then drop the sender
            let _ = done_tx.send(());
        });

        // Convert receiver to stream
        let log_stream = UnboundedReceiverStream::new(log_rx);
        let stream = log_stream.take_until(done_rx);
        Ok(Response::new(Box::pin(stream)))
    }
}
