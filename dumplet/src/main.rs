use clap::Parser;
use dumplet::export_docker_image;

#[derive(Parser, Debug)]
#[command(author, version, about = "Export a Docker image’s filesystem as a tar archive", long_about = None)]
struct Args {
    #[arg(help = "The name of the Docker image (e.g. ubuntu:latest)")]
    image: String,

    #[arg(help = "The output file path for the exported tar archive")]
    output: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = export_docker_image(&args.image, &args.output).await {
        eprintln!("Erreur : {}", e);
        std::process::exit(1);
    }
}
