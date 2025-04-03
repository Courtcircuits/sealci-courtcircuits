use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// The host URL of the scheduler
    #[clap(long, default_value = "http://[::1]:50051")]
    pub shost: String,

    /// The host URL you want the scheduler to contact the agent on
    #[clap(long, default_value = "http://[::1]")]
    pub ahost: String,

    /// The port of the agent to listen on
    #[clap(long, default_value = "9001")]
    pub port: u32,
}
