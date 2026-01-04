use clap::Parser;

use kube_podlog::cli::Cli;
use kube_podlog::config::Config;

#[tokio::main]
async fn main() -> kube_podlog::errors::AppResult<()> {
    kube_podlog::logging::init();

    let cli = Cli::parse();
    let config: Config = cli.into(); // <-- infallible conversion, no `?`

    kube_podlog::run(config).await
}
