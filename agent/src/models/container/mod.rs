use std::{sync::Arc, time::Duration};
use tokio::{task, time::sleep};
pub mod exec_handle;
use bollard::{
    container::Config,
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};
use exec_handle::ExecResult;
use futures_util::TryStreamExt;

use super::error::Error::{
    self, ContainerExecDetachedError, ContainerExecError, ContainerRemoveError,
    ContainerStartError, PullImageError,
};

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub config: Config<String>,
    docker: Arc<Docker>,
}

impl Container {
    pub fn new(image: String, docker: Arc<Docker>) -> Self {
        let id = format!("{:x}", rand::random::<u128>());
        let entrypoint = Some(vec!["/bin/sh".to_string()]);
        let config = Config {
            entrypoint,
            image: Some(image),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };
        Container { id, config, docker }
    }

    pub async fn start(&self) -> Result<(), Error> {
        // Get the image
        self.docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: self.config.image.clone().unwrap_or("debian".to_string()),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .map_err(PullImageError)?;

        self.docker
            .create_container::<String, String>(
                Some(bollard::container::CreateContainerOptions {
                    name: self.id.clone(),
                    platform: None,
                }),
                self.config.clone(),
            )
            .await
            .map_err(ContainerStartError)?;
        self.docker
            .start_container::<String>(&self.id, None)
            .await
            .map_err(ContainerStartError)?;
        Ok(())
    }

    pub async fn exec(
        &self,
        command: String,
        workdir: Option<String>,
    ) -> Result<ExecResult, Error> {
        let exec = self
            .docker
            .create_exec(
                &self.id,
                CreateExecOptions {
                    cmd: Some(command.split(' ').map(String::from).collect()),
                    tty: Some(true),
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    working_dir: workdir,
                    ..Default::default()
                },
            )
            .await
            .map_err(ContainerExecError)?;
        let exec_result: StartExecResults = self
            .docker
            .start_exec(exec.id.as_str(), None)
            .await
            .map_err(ContainerExecError)?;
        let docker = self.docker.clone();

        // Check asyncly for the status of the exec task
        let exec_handle = task::spawn(async move {
            loop {
                let exec_state = match docker.inspect_exec(&exec.id).await {
                    Ok(exec_state) => exec_state,
                    Err(_) => return 1,
                };
                match exec_state.exit_code {
                    Some(exit_code) => return exit_code as i32,
                    _ => {}
                }
                match exec_state.running {
                    Some(true) => {
                        sleep(Duration::from_secs(1)).await;
                    }
                    _ => return 1,
                }
            }
        });

        // The stream of stdout of the exec
        let output = match exec_result {
            StartExecResults::Attached { output, input: _ } => output,
            StartExecResults::Detached => return Err(ContainerExecDetachedError),
        };

        Ok(ExecResult {
            output,
            exec_handle,
        })
    }

    pub async fn remove(&self) -> Result<(), Error> {
        self.docker
            .remove_container(&self.id, None)
            .await
            .map_err(ContainerRemoveError)?;
        Ok(())
    }
}
