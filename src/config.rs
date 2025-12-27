use crate::cli::{Cli, ColorBy, ColorMode};
use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct Config {
    pub namespace: String,
    pub selector: String,

    pub all_containers: bool,
    pub container: Option<String>,

    pub output: OutputMode,
    pub timestamps: bool,

    pub color: ColorMode,
    pub color_by: ColorBy,

    pub buffer: usize,
    pub max_concurrency: Option<usize>,

    pub verbosity: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputMode {
    Human,
    Json,
}

impl TryFrom<Cli> for Config {
    type Error = AppError;

    fn try_from(cli: Cli) -> AppResult<Self> {
        if cli.selector.trim().is_empty() {
            return Err(AppError::Cli("label selector cannot be empty".into()));
        }

        if cli.buffer == 0 {
            return Err(AppError::Cli("--buffer must be > 0".into()));
        }

        if let Some(0) = cli.max_concurrency {
            return Err(AppError::Cli("--max-concurrency must be > 0".into()));
        }

        // Normalize booleans that have "no-*" companions
        let all_containers = if cli.no_all_containers {
            false
        } else {
            cli.all_containers
        };

        let timestamps = if cli.no_timestamps {
            false
        } else {
            cli.timestamps
        };

        // Container flag overrides all-containers
        let all_containers = if cli.container.is_some() {
            false
        } else {
            all_containers
        };

        let namespace = cli.namespace.unwrap_or_else(|| "default".to_string());

        Ok(Config {
            namespace,
            selector: cli.selector,

            all_containers,
            container: cli.container,

            output: if cli.json {
                OutputMode::Json
            } else {
                OutputMode::Human
            },
            timestamps,

            color: cli.color,
            color_by: cli.color_by,

            buffer: cli.buffer,
            max_concurrency: cli.max_concurrency,

            verbosity: cli.verbose,
        })
    }
}
