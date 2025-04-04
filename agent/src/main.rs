use agent::{app::App, models::error::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let mut app = App::init().await?;
    app.start().await?;
    Ok(())
}
