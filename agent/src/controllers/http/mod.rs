use std::sync::Arc;

use action_dto::ActionDto;
use actix_web::{
    error::{self},
    get, rt, web, Error, HttpRequest, HttpResponse,
};
use futures::StreamExt;
use tokio::sync::Mutex;
use tracing::info;

use crate::services::action_service::ActionService;
mod action_dto;

#[derive(Clone)]
pub struct ActionController {
    pub action_service: Arc<Mutex<ActionService>>,
}

impl ActionController {
    pub fn new(action_service: Arc<Mutex<ActionService>>) -> Self {
        Self { action_service }
    }

    pub fn register(self, config: &mut web::ServiceConfig) {
        let service = Arc::new(self);

        config.app_data(web::Data::new(service.action_service.clone()));
        config.service(
            web::scope("/actions")
                .service(list_actions)
                .service(stream_actions), // .service(Self::state_events_ws),
                                          // .service(Self::get_action)
                                          // .service(Self::create_action)
                                          // .service(Self::delete_action)
        );
    }
}

#[get("")]
async fn list_actions(
    service: web::Data<Arc<Mutex<ActionService>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let actions = service
        .lock()
        .await
        .list()
        .await
        .map_err(|e| error::ErrorPreconditionFailed(e.to_string()))?;

    // Convert to DTOs
    let action_dtos: Vec<ActionDto> = actions
        .into_iter()
        .map(|action| ActionDto {
            id: action.id,
            state: format!("{:?}", action.state),
            repo_url: action.repository_url.clone(),
            image: action
                .container
                .config
                .image
                .clone()
                .unwrap_or("No image found".to_string()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(action_dtos))
}

#[get("/stream")]
async fn stream_actions(
    service: web::Data<Arc<Mutex<ActionService>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut actions_stream = service.lock().await.creation_stream();

    let (res, mut session, _) = match actix_ws::handle(&req, stream) {
        Ok(result) => result,
        Err(err) => {
            tracing::info!("Failed to handle WebSocket connection: {}", err);
            return Err(err);
        }
    };
    info!("here");

    rt::spawn(async move {
        while let Some(action) = actions_stream.next().await {
            let action = match action {
                Some(action) => action,
                None => continue,
            };
            let action_dto = ActionDto {
                id: action.id,
                state: format!("{:?}", action.state),
                repo_url: action.repository_url.clone(),
                image: action
                    .container
                    .config
                    .image
                    .clone()
                    .unwrap_or("No image found".to_string()),
            };
            let action_json = match serde_json::to_string(&action_dto) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!("Failed to serialize action DTO: {}", err);
                    continue;
                }
            };

            if let Err(err) = session.text(action_json).await {
                tracing::error!("Failed to send action ID: {}", err);
            }
        }
    });
    Ok(res)
}
