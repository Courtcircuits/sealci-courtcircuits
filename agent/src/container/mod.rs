use containerd_client::services::v1::{container::Runtime, Container as ContainerD, CreateContainerRequest};
use image::{Image, Pull};

use crate::client_container::ClientContainer;
use std::error::Error;
pub mod image;
pub mod old;

pub struct Container {
    client: ClientContainer,
    image: Image,
    id: String,
}

pub trait Manager {
    async fn launch(&self) -> Result<(), Box<dyn Error>>;
    async fn stop(&self) -> Result<(), Box<dyn Error>>;
    async fn remove(&self) -> Result<(), Box<dyn Error>>;
    async fn exec(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Builder {
    async fn build(
        image_name: String,
        client: ClientContainer,
    ) -> Result<Container, Box<dyn Error>>;
}

impl Builder for Container {
    async fn build(
        image_name: String,
        client: ClientContainer,
    ) -> Result<Container, Box<dyn Error>> {
        let image = Image::new(image_name, client.clone());
        image.clone().pull().await?;
        Ok(Container { client, image, id: "".to_string() })
    }
}

impl Manager for Container {
    async fn launch(&self) -> Result<(), Box<dyn Error>> {
            // let spec = include_str!("container_spec.json");
            // let spec = spec
            //         .to_string()
            //         .replace("$ROOTFS", rootfs)
            //         .replace("$OUTPUT", output);
            
            //     let spec = Any {
            //         type_url: "types.containerd.io/opencontainers/runtime-spec/1/Spec".to_string(),
            //         value: spec.into_bytes(),
            //     };
        let container = ContainerD::
        let request = CreateContainerRequest {
            container: Some(container),
        };
        self.client.containers.create(request);
        Ok(())
        
    }

    async fn stop(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    async fn remove(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    async fn exec(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
