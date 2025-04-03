use tokio::task::JoinError;

#[allow(dead_code)]
pub enum Error {
    // Define your custom error variants here
    ContainerStartError(bollard::errors::Error),
    PullImageError(bollard::errors::Error),
    ContainerRemoveError(bollard::errors::Error),
    ContainerExecError(bollard::errors::Error),
    ContainerExecDetachedError,
    ExecError(JoinError),
    StepOutputError(i32),
}
