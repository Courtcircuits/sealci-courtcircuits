use std::sync::Arc;

use action_dto::{ActionDto, DeleteActionResponse};
use actix_web::{
    delete,
    error::{self},
    get, post, rt, web, Error, HttpRequest, HttpResponse,
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
                .service(stream_actions)
                .service(stream_action_state)
                .service(list_actions)
                .service(get_action)
                .service(create_action)
                .service(delete_action),
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
    let mut action_dtos: Vec<ActionDto> = Vec::new();
    for action in actions {
        let state = action.state.read().await.to_owned();
        action_dtos.push(ActionDto {
            id: action.id,
            state,
            repo_url: action.repository_url.clone(),
            image: action
                .container
                .config
                .image
                .clone()
                .unwrap_or("No image found".to_string()),
        });
    }

    Ok(HttpResponse::Ok().json(action_dtos))
}

#[post("")]
async fn create_action(
    service: web::Data<Arc<Mutex<ActionService>>>,
    req: web::Json<action_dto::CreateActionRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Create a dummy channel for logs (real logs go through gRPC)
    let (log_tx, _) = tokio::sync::mpsc::unbounded_channel();

    let action = service
        .lock()
        .await
        .create(
            req.image.clone(),
            req.commands.clone(),
            log_tx,
            req.repo_url.clone(),
            req.action_id,
        )
        .await
        .map_err(|e| error::ErrorPreconditionFailed(e.to_string()))?;
    let action_state = action.state.read().await;
    let action_dto = ActionDto {
        id: action.id,
        state: action_state.to_owned(),
        repo_url: action.repository_url.clone(),
        image: action
            .container
            .config
            .image
            .clone()
            .unwrap_or("No image found".to_string()),
    };
    Ok(HttpResponse::Created().json(action_dto))
}

#[get("/{id}")]
async fn get_action(
    service: web::Data<Arc<Mutex<ActionService>>>,
    path: web::Path<u32>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();
    let action = service
        .lock()
        .await
        .get(id)
        .await
        .map_err(|e| error::ErrorPreconditionFailed(e.to_string()))?;
    let action_state = action.state.read().await.to_string();

    info!("{}", action_state);
    let action_dto = ActionDto {
        id: action.id,
        state: action.state.read().await.to_owned(),
        repo_url: action.repository_url.clone(),
        image: action
            .container
            .config
            .image
            .clone()
            .unwrap_or("No image found".to_string()),
    };
    Ok(HttpResponse::Created().json(action_dto))
}

#[delete("/{id}")]
async fn delete_action(
    service: web::Data<Arc<Mutex<ActionService>>>,
    path: web::Path<u32>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();
    service
        .lock()
        .await
        .delete(id)
        .await
        .map_err(|e| error::ErrorPreconditionFailed(e.to_string()))?;
    Ok(HttpResponse::Created().json(DeleteActionResponse { id }))
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

    rt::spawn(async move {
        while let Some(action) = actions_stream.next().await {
            let action = match action {
                Some(action) => action,
                None => continue,
            };
            let action_state = action.state.read().await;
            let action_dto = ActionDto {
                id: action.id,
                state: action_state.to_owned(),
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

#[get("/state")]
async fn stream_action_state(
    service: web::Data<Arc<Mutex<ActionService>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut state_stream = service.lock().await.state_stream();

    let (res, mut session, _) = match actix_ws::handle(&req, stream) {
        Ok(result) => result,
        Err(err) => {
            tracing::info!("Failed to handle WebSocket connection: {}", err);
            return Err(err);
        }
    };

    rt::spawn(async move {
        while let Some(action) = state_stream.next().await {
            let state = match action {
                Some(state) => state,
                None => continue,
            };

            let action_json = match serde_json::to_string(&state) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!("Failed to serialize state: {}", err);
                    continue;
                }
            };

            if let Err(err) = session.text(action_json).await {
                tracing::error!("Failed to send state of action ID: {}", err);
            }
        }
    });
    Ok(res)
}
