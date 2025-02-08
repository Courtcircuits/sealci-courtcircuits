use crate::client_container::ClientContainer;
use containerd_client::{
    services::v1::{
        container::Runtime, Container as ContainerD, CreateContainerRequest, DeleteContainerRequest,
    },
    with_namespace,
};
use image::{Image, Pull, Unpack};
use log::info;
use prost_types::Any;
use std::{error::Error, path::PathBuf};
use tonic::Request;
pub mod image;
pub mod old;

const NAMESPACE: &str = "default";

pub struct Container {
    client: ClientContainer,
    image: Image,
    id: String,
    rootfs: PathBuf,
}

pub trait Manager: Send + Sync {
    async fn create(&self) -> Result<(), Box<dyn Error>>;
    async fn remove(self) -> Result<(), Box<dyn Error>>;
    async fn exec(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Builder {
    async fn build(image: Image) -> Result<Container, Box<dyn Error>>;
}

impl Builder for Container {
    async fn build(image: Image) -> Result<Container, Box<dyn Error>> {
        image.clone().pull().await?;
        // Define the directory where the image will be unpacked
        let unpack_dir = PathBuf::from(format!("/tmp/containerd/unpacked/{}", image.name));
        std::fs::create_dir_all(&unpack_dir)?;

        // Unpack the image
        let id = rand::random::<u64>().to_string();
        Ok(Container {
            client: image.client.clone(),
            image,
            id,
            rootfs: unpack_dir,
        })
    }
}

impl Manager for Container {
    async fn create(&self) -> Result<(), Box<dyn Error>> {
        info!(
            "Creating container {} with image {}",
            self.id, self.image.name
        );
        let spec = include_str!("container_spec.json");
        let spec = spec.to_string().replace("$ROOTFS", "rootfs");
        let spec = Any {
            type_url: "types.containerd.io/opencontainers/runtime-spec/1/Spec".to_string(),
            value: spec.into_bytes(),
        };
        let container = ContainerD {
            id: self.id.clone(),
            image: self.image.name.clone(),
            runtime: Some(Runtime {
                name: "io.containerd.runc.v2".to_string(),
                options: None,
            }),
            spec: Some(spec),
            ..Default::default()
        };
        let request = CreateContainerRequest {
            container: Some(container),
        };
        let request = with_namespace!(request, NAMESPACE);
        info!("Launching request for container {}", self.id);
        self.client.containers.clone().create(request).await?;
        Ok(())
    }

    async fn remove(mut self) -> Result<(), Box<dyn Error>> {
        let req = DeleteContainerRequest { id: self.id };
        let req = with_namespace!(req, NAMESPACE);
        let _resp = self.client.containers.delete(req).await?;
        Ok(())
    }

    async fn exec(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
