use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "kube-podlog",
    version,
    about = "Fast multi-pod Kubernetes log tailer (WIP)"
)]
pub struct Cli {
    /// Kubernetes namespace (default: "default")
    #[arg(short = 'n', long = "namespace")]
    pub namespace: Option<String>,

    /// Label selector (e.g. app=web,tier=frontend)
    #[arg(short = 'l', long = "selector")]
    pub selector: String,

    /// Run in dev mode (no Kubernetes cluster required)
    #[arg(long = "dev", default_value_t = false)]
    pub dev: bool,

    /// Emit newline-delimited JSON (NDJSON) instead of human output
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,

    /// Disable timestamps in human output
    #[arg(long = "no-timestamps", default_value_t = false)]
    pub no_timestamps: bool,

    /// Disable colored output (human mode only)
    #[arg(long = "no-color", default_value_t = false)]
    pub no_color: bool,

    /// Colorize by "pod" or "container" (human mode only)
    #[arg(long = "color-by", value_enum, default_value_t = ColorByArg::Pod)]
    pub color_by: ColorByArg,

    /// Bounded channel size for merged log events (backpressure safety)
    #[arg(long = "buffer", default_value_t = 1024)]
    pub buffer: usize,

    /// Dev mode: milliseconds between log lines per stream
    #[arg(long = "dev-rate-ms", default_value_t = 500)]
    pub dev_rate_ms: u64,

    /// Dev mode: max number of lines per stream (useful for tests)
    #[arg(long = "dev-lines")]
    pub dev_lines: Option<u64>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorByArg {
    Pod,
    Container,
}
