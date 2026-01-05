use crate::cli::Cli;
use crate::types::{ColorMode, OutputConfig, OutputMode};

#[derive(Debug, Clone)]
pub struct RuntimeOpts {
    pub buffer: usize,
}

#[derive(Debug, Clone)]
pub struct DevOpts {
    pub rate_ms: u64,
    pub lines: u64,
}

#[derive(Debug, Clone)]
pub struct KubeLogOpts {
    pub containers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub namespace: String,
    pub selector: String,
    pub dev_mode: bool,

    pub output: OutputConfig,
    pub runtime: RuntimeOpts,
    pub dev: DevOpts,
    pub kube: KubeLogOpts,
}

impl TryFrom<Cli> for Config {
    type Error = std::convert::Infallible;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let mode = if cli.json {
            OutputMode::Json
        } else {
            OutputMode::Human
        };

        let color = if cli.json {
            ColorMode::Never
        } else {
            cli.color.into()
        };

        Ok(Config {
            namespace: cli.namespace,
            selector: cli.selector,
            dev_mode: cli.dev,
            output: OutputConfig {
                mode,
                color_by: cli.color_by.into(),
                color,
                no_color: cli.no_color,
            },
            runtime: RuntimeOpts { buffer: 2048 },
            dev: DevOpts {
                rate_ms: cli.dev_rate_ms,
                lines: cli.dev_lines,
            },
            kube: KubeLogOpts {
                containers: Vec::new(),
            },
        })
    }
}
