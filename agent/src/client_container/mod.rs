// TransferClient<Channel>

use std::error::Error;

use containerd_client;
use containerd_client::services::v1::containers_client::ContainersClient;
use containerd_client::services::v1::images_client::ImagesClient;
use containerd_client::services::v1::transfer_client::TransferClient;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct ClientContainer {
    pub raw: TransferClient<Channel>,
    pub containers: ContainersClient<Channel>,
    pub images: ImagesClient<Channel>,
}


impl ClientContainer {
    pub async fn new(socket: Option<String>) -> Result<Self, Box<dyn Error>> {
        let socket = socket.unwrap_or("/run/containerd/containerd.sock".to_string());
        let channel = containerd_client::connect(socket).await?;
        let raw = TransferClient::new(channel.clone());
        let containers = ContainersClient::new(channel.clone());
        let images = ImagesClient::new(channel.clone());
        Ok(Self {
            raw,
            containers,
            images,
        })
    }
}
