

use derive_more::Display;
use tokio::task::JoinError;
use tonic::Status;

#[allow(dead_code)]
#[derive(Debug,Display)]
pub enum Error {
    // Define your custom error variants here\
    Error(String),
    DockerConnectionError(bollard::errors::Error),
    ContainerStartError(bollard::errors::Error),
    PullImageError(bollard::errors::Error),
    ContainerRemoveError(bollard::errors::Error),
    ContainerExecError(bollard::errors::Error),
    ContainerExecDetachedError,
    ExecError(JoinError),
    StepOutputError(i32),
    ConnectionError(tonic::transport::Error),
    ServeError(tonic::transport::Error),
    RegistrationError(Status),
    ReportHealthError(Status),
    NotRegisteredError,
    HealthStreamError,
    ActionNotFound,
    ActionStateError,
    BrokerSendError(String),
    ChannelError(String),
}
