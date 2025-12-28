use clap::{Parser, ValueEnum};

/// kube-podlog: fast multi-pod Kubernetes log tailer.
#[derive(Debug, Clone, Parser)]
#[command(name = "kube-podlog", version, about)]
pub struct Cli {
    /// Namespace to search in (defaults to current context namespace; fallback: "default")
    #[arg(short = 'n', long = "namespace")]
    pub namespace: Option<String>,

    /// Label selector, e.g. app=web,tier=frontend
    #[arg(short = 'l', long = "selector")]
    pub selector: String,

    /// Run in dev mode (no Kubernetes cluster required)
    #[arg(long = "dev", default_value_t = false)]
    pub dev: bool,

    /// Stream logs from all containers in each pod (default: true)
    #[arg(long = "all-containers", default_value_t = true)]
    pub all_containers: bool,

    /// Disable the default all-containers behavior
    #[arg(long = "no-all-containers", default_value_t = false)]
    pub no_all_containers: bool,

    /// Only stream a specific container (overrides all-containers)
    #[arg(short = 'c', long = "container")]
    pub container: Option<String>,

    /// Emit newline-delimited JSON objects (NDJSON)
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,

    /// Prefix each line with an RFC3339 timestamp (default: true)
    #[arg(long = "timestamps", default_value_t = true)]
    pub timestamps: bool,

    /// Disable timestamps
    #[arg(long = "no-timestamps", default_value_t = false)]
    pub no_timestamps: bool,

    /// Color mode for output
    #[arg(long = "color", value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    /// Assign colors by pod or container
    #[arg(long = "color-by", value_enum, default_value_t = ColorBy::Pod)]
    pub color_by: ColorBy,

    /// Global bounded event buffer size (backpressure-safe)
    #[arg(long = "buffer", default_value_t = 2048)]
    pub buffer: usize,

    /// Optional cap on number of concurrent log streams
    #[arg(long = "max-concurrency")]
    pub max_concurrency: Option<usize>,

    /// Increase verbosity (-v, -vv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorBy {
    Pod,
    Container,
}
