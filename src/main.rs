use kube_podlog::{cli::Cli, config::Config, errors::AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    kube_podlog::tracing::init();

    let cli = <Cli as clap::Parser>::parse();
    let config = Config::try_from(cli)?;

    kube_podlog::run(config).await
}
