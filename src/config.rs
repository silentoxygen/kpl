use crate::cli::{Cli, ColorByArg, TimestampSourceArg};
use crate::types::{
    ColorBy, DevOpts, KubeLogOpts, OutputConfig, OutputMode, RuntimeOpts, TimestampSource,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub namespace: String,
    pub selector: String,
    pub dev_mode: bool,

    pub dev: DevOpts,
    pub kube: KubeLogOpts,
    pub runtime: RuntimeOpts,
    pub output: OutputConfig,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let mode = if cli.json {
            OutputMode::Json
        } else {
            OutputMode::Human
        };

        let color_by = match cli.color_by {
            ColorByArg::Pod => ColorBy::Pod,
            ColorByArg::Container => ColorBy::Container,
        };

        let ts_source = match cli.timestamps_source {
            TimestampSourceArg::Local => TimestampSource::Local,
            TimestampSourceArg::Kube => TimestampSource::Kube,
        };

        Self {
            namespace: cli.namespace,
            selector: cli.selector,
            dev_mode: cli.dev,

            dev: DevOpts {
                rate_ms: cli.dev_rate_ms,
                lines: cli.dev_lines,
            },

            kube: KubeLogOpts {
                containers: cli.container,
                since_seconds: cli.since_seconds,
                tail_lines: cli.tail,
                timestamps_source: ts_source,
                reconnect_min_ms: cli.reconnect_min_ms,
                reconnect_max_ms: cli.reconnect_max_ms,
            },

            runtime: RuntimeOpts { buffer: cli.buffer },

            output: OutputConfig {
                mode,
                color: !cli.no_color,
                color_by,
                timestamps: !cli.no_timestamps,
            },
        }
    }
}
