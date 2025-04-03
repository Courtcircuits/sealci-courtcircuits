use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bollard::exec::{CreateExecResults, StartExecResults};
use futures_util::StreamExt;
use tokio::{spawn, sync::mpsc::UnboundedSender, task, time::sleep};
use tonic::Status;
use tracing::info;
use url::Url;

use crate::{
    container::{
        create_exec, inspect_exec, launch_container, remove_container, start_exec, stop_container,
    },
    proto::{ActionResponseStream, ActionResult},
};

pub async fn launch_action(
    image_name: String,
    commands: &mut Vec<String>,
    log_input: Arc<Mutex<UnboundedSender<Result<ActionResponseStream, Status>>>>,
    repo_url: String,
    action_id: Arc<Mutex<u32>>,
) -> Result<(), Status> {
    let container_id: String = match launch_container(&image_name).await {
        Ok(id) => id,
        Err(e) => return Err(Status::aborted(format!("Launching error: {}", e))),
    };

    let repo_name = setup_repository(repo_url, container_id.as_str()).await?;

    for command in &mut *commands {
        let log_input = Arc::clone(&log_input);
        let action_id = Arc::clone(&action_id);
        let absolute_path = format!("/{}", repo_name);
        let exec_id = start_command(    
            command,
            &container_id,
            log_input.clone(),
            Some(absolute_path),
            action_id.clone(),
        )
        .await?;
        match wait_for_command(exec_id, &container_id).await {
            Ok(_) => info!("Command completed"),
            Err(e) => {
                let _ = log_input.lock().unwrap().send(Ok(ActionResponseStream {
                    log: format!("Error happened: {}", e),
                    action_id: *action_id.lock().unwrap(),
                    result: Some(ActionResult {
                        completion: 3,
                        exit_code: Some(1),
                    }),
                }));
            }
        }
    }
    Ok(())
}

pub async fn setup_repository(repo_url: String, container_id: &str) -> Result<String, Status> {
    let setup_command = format!("git clone {}", repo_url);
    let exec_id = match create_exec(&setup_command, container_id, None).await {
        Ok(CreateExecResults { id }) => id,
        Err(_) => return Err(Status::aborted("Error happened when creating exec")),
    };
    let _ = start_exec(&exec_id).await;
    wait_for_command(exec_id, &container_id.to_string()).await?;
    let repo_name = match get_repo_name(&repo_url) {
        Some(repo_name) => Ok(repo_name),
        None => Err(Status::aborted("Error happened when getting repo name")),
    };
    repo_name
}

pub async fn start_command(
    command: &mut String,
    container_id: &str,
    log_input: Arc<Mutex<UnboundedSender<Result<ActionResponseStream, Status>>>>,
    repo_name: Option<String>,
    action_id: Arc<Mutex<u32>>,
) -> Result<String, Status> {
    let mut container_ouput = match start_exec("").await {
        Ok(StartExecResults::Attached { output, input: _ }) => output,
        Ok(StartExecResults::Detached) => return Err(Status::aborted("Can't attach to container")),
        Err(_) => return Err(Status::aborted("Error happened when launching action")),
    };
    task::spawn(async move {
        while let Some(log) = container_ouput.next().await {
            let container_log_output = match log {
                Ok(log_output) => log_output,
                Err(e) => return Err(Status::aborted(format!("Execution error: {}", e))),
            };
            let _ = &log_input.lock().unwrap().send(Ok(ActionResponseStream {
                log: container_log_output.to_string(),
                action_id: *action_id.lock().unwrap(),
                result: Some(ActionResult {
                    completion: 2,
                    exit_code: None,
                }),
            }));
        }
        Ok(())
    });
    Ok(exec_id)
}

pub async fn wait_for_command(exec_id: String) -> Result<(), Status> {
    loop {
        let exec_state = match inspect_exec(&exec_id).await {
            Ok(exec_state) => exec_state,
            Err(_) => return Err(Status::aborted("Error happened checking state of a step")),
        };
        match exec_state.exit_code {
            Some(0) => {
                break;
            }
            Some(exit_code) => {
                return Err(Status::aborted("Step exited with an error"));
            }
            None => {}
        }
        match exec_state.running {
            Some(true) => {}
            Some(false) => {
                break;
            }
            None => {
                return Err(Status::aborted("Error happened checking state of a step"));
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

fn get_repo_name(github_url: &str) -> Option<String> {
    let url = Url::parse(github_url).ok()?;
    let segments: Vec<&str> = url.path_segments()?.collect();
    segments.last().map(|s| s.to_string())
}
