use crate::client_container::ClientContainer;

use containerd_client::{
    services::v1::{TransferOptions, TransferRequest},
    to_any,
    types::{
        transfer::{ImageStore, OciRegistry, UnpackConfiguration},
        Platform,
    },
    with_namespace,
};
use std::{env::consts, error::Error, path::PathBuf};
use tonic::Request;
use tracing::info;
const NAMESPACE: &str = "default";

#[derive(Clone)]
pub struct Image {
    pub name: String,
    pub client: ClientContainer,
}

pub trait Pull {
    async fn pull(self) -> Result<(), Box<dyn Error>>;
}

pub trait Unpack {
    async fn unpack(self, unpack_dir: PathBuf) -> Result<(), Box<dyn Error>>;
}

impl Unpack for Image {
    async fn unpack(mut self, unpack_dir: PathBuf) -> Result<(), Box<dyn Error>> {
        info!("Unpacking image: {}", self.name);
        let platform = Platform {
            os: "linux".to_string(),
            architecture: consts::ARCH.to_string(),
            variant: "".to_string(),
            os_version: "".to_string(),
        };

        let unpack_config = UnpackConfiguration {
            platform: Some(platform),
            snapshotter: unpack_dir.to_str().unwrap().to_string(),
            ..Default::default()
        };

        let request = TransferRequest {
            source: Some(to_any(&ImageStore {
                name: self.name.clone(),
                ..Default::default()
            })),
            destination: Some(to_any(&unpack_config)),
            options: Some(TransferOptions {
                ..Default::default()
            }),
        };

        self.client
            .raw
            .transfer(with_namespace!(request, NAMESPACE))
            .await
            .expect("unable to unpack image");

        info!("Image unpacked to: {:?}", unpack_dir);
        Ok(())
    }
}

impl Pull for Image {
    async fn pull(mut self) -> Result<(), Box<dyn Error>> {
        info!("Pulling image: {}", self.name);
        let arch = match consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => consts::ARCH,
        };

        // Create the source (OCIRegistry)
        let source = OciRegistry {
            reference: self.name.clone(),
            resolver: Default::default(),
        };

        let platform = Platform {
            os: "linux".to_string(),
            architecture: arch.to_string(),
            variant: "".to_string(),
            os_version: "".to_string(),
        };

        // Create the destination (ImageStore)
        let destination = ImageStore {
            name: self.name.clone(),
            platforms: vec![platform.clone()],
            unpacks: vec![UnpackConfiguration {
                platform: Some(platform),
                ..Default::default()
            }],
            ..Default::default()
        };

        let anys = to_any(&source);
        let anyd = to_any(&destination);

        info!("Pulling image for linux/{} from source: {:?}", arch, source);

        // Create the transfer request
        let request = TransferRequest {
            source: Some(anys),
            destination: Some(anyd),
            options: Some(TransferOptions {
                ..Default::default()
            }),
        };
        // Execute the transfer (pull)
        self.client
            .raw
            .transfer(with_namespace!(request, NAMESPACE))
            .await
            .expect("unable to transfer image");
        info!("Image name: {}", self.name);
        Ok(())
    }
}

impl Image {
    pub fn new(name: String, client: ClientContainer) -> Self {
        Image { name, client }
    }
}
