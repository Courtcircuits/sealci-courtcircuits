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

pub trait ContainerManager {
    fn launch_container(&self) -> Result<Container, Box<dyn Error>>;
    fn stop_container(&self) -> Result<Container, Box<dyn Error>>;
    fn remove_container(&self) -> Result<(), Box<dyn Error>>;
    fn launch_exec(&self) -> Result<(), Box<dyn Error>>;
}

pub trait ContainerBuilder<T>
where
    T: Pull,
{
    fn build(image: T) -> Result<Container, Box<dyn Error>>;
}
