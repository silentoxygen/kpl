use clap::Parser;

use kpl::cli::Cli;
use kpl::config::Config;

#[tokio::main]
async fn main() -> kpl::errors::AppResult<()> {
    kpl::logging::init();

    let cli = Cli::parse();
    let config = Config::try_from(cli).expect("infallible config conversion failed");

    kpl::run(config).await
}
