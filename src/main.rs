use clap::Parser;

use kube_podlog::cli::Cli;
use kube_podlog::config::Config;

#[tokio::main]
async fn main() -> kube_podlog::errors::AppResult<()> {
    kube_podlog::logging::init();

    let cli = Cli::parse();
    let config = Config::try_from(cli).expect("infallible config conversion failed");

    kube_podlog::run(config).await
}
