use tokio::task::JoinError;

pub enum Error {
    // Define your custom error variants here
    ContainerStartError(bollard::errors::Error),
    PullImageError(bollard::errors::Error),
    ContainerRemoveError(bollard::errors::Error),
    ContainerExecError(bollard::errors::Error),
    ContainerExecDetachedError,
    StepError(JoinError),
    StepOutputError
}
