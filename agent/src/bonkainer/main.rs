use agent::{
    client_container::ClientContainer,
    container::{image::Image, Builder, Container, Manager},
};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let client = ClientContainer::new(None).await?;
    let image = Image::new(
        "docker.io/milou666/beep-front:a40a93c2-staging".to_string(),
        client.clone(),
    );
    let container = Container::build(image).await?;
    container.create().await?;
    // container.remove().await?;
    Ok(())
}
