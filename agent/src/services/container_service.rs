use std::sync::Arc;

use bollard::Docker;

use crate::models::{container::Container, error::Error};

pub struct ContainerService {
    docker_client: Arc<Docker>,
}

impl ContainerService {
    pub fn new(docker_client: Arc<Docker>) -> Self {
        Self { docker_client }
    }

    pub async fn create_container(&self, image: String) -> Result<Container, Error> {
        let container = Container::new(image, self.docker_client.clone());
        container.start().await?;
        Ok(container)
    }
}
