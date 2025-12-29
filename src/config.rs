use std::io::IsTerminal;

use crate::cli::{Cli, ColorByArg};
use crate::errors::{AppError, AppResult};

#[derive(Clone, Copy, Debug)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug)]
pub enum ColorBy {
    Pod,
    Container,
}

#[derive(Clone, Copy, Debug)]
pub struct HumanFormat {
    pub timestamps: bool,
    pub color: bool,
    pub color_by: ColorBy,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub human: HumanFormat,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub namespace: String,
    pub selector: String,
    pub dev_mode: bool,

    pub output: OutputConfig,
    pub buffer: usize,

    pub dev_rate_ms: u64,
    pub dev_lines: Option<u64>,
}

impl TryFrom<Cli> for Config {
    type Error = AppError;

    fn try_from(cli: Cli) -> AppResult<Self> {
        let namespace = cli.namespace.unwrap_or_else(|| "default".to_string());

        let mode = if cli.json {
            OutputMode::Json
        } else {
            OutputMode::Human
        };

        let color_by = match cli.color_by {
            ColorByArg::Pod => ColorBy::Pod,
            ColorByArg::Container => ColorBy::Container,
        };

        // Default: enable color only if stdout is a terminal and user didn't disable it.
        let stdout_is_tty = std::io::stdout().is_terminal();
        let enable_color = !cli.no_color && stdout_is_tty;

        let output = OutputConfig {
            mode,
            human: HumanFormat {
                timestamps: !cli.no_timestamps,
                color: enable_color,
                color_by,
            },
        };

        Ok(Self {
            namespace,
            selector: cli.selector,
            dev_mode: cli.dev,

            output,
            buffer: cli.buffer,

            dev_rate_ms: cli.dev_rate_ms,
            dev_lines: cli.dev_lines,
        })
    }
}
