use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorByArg {
    Pod,
    Container,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TimestampSourceArg {
    Local,
    Kube,
}

#[derive(Debug, Parser)]
#[command(
    name = "kube-podlog",
    version,
    about = "Fast multi-pod Kubernetes log tailer"
)]
pub struct Cli {
    /// Namespace (defaults to "default")
    #[arg(short = 'n', long, default_value = "default")]
    pub namespace: String,

    /// Label selector (e.g. app=web,tier=frontend)
    #[arg(short = 'l', long, required = true)]
    pub selector: String,

    /// Dev mode (no cluster required). Uses simulated pods/logs.
    #[arg(long, default_value_t = false)]
    pub dev: bool,

    /// Dev mode: delay between lines per stream
    #[arg(long, default_value_t = 500)]
    pub dev_rate_ms: u64,

    /// Dev mode: max lines per container (None = infinite)
    #[arg(long)]
    pub dev_lines: Option<u64>,

    /// Channel buffer for log fan-in (backpressure control)
    #[arg(long, default_value_t = 1024)]
    pub buffer: usize,

    /// Output: newline-delimited JSON objects
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Disable ANSI color
    #[arg(long, default_value_t = false)]
    pub no_color: bool,

    /// Color assignment strategy
    #[arg(long, value_enum, default_value_t = ColorByArg::Pod)]
    pub color_by: ColorByArg,

    /// Disable timestamps in human mode
    #[arg(long, default_value_t = false)]
    pub no_timestamps: bool,

    // ---- Kube log options ----
    /// Limit logs to these container(s). Repeatable. If not set => all containers (default).
    #[arg(long = "container")]
    pub container: Vec<String>,

    /// Only return logs newer than this many seconds.
    #[arg(long)]
    pub since_seconds: Option<i64>,

    /// Lines of recent log file to display (like kubectl --tail).
    #[arg(long)]
    pub tail: Option<i64>,

    /// Timestamp source used in output. (local = time we received the line)
    #[arg(long, value_enum, default_value_t = TimestampSourceArg::Local)]
    pub timestamps_source: TimestampSourceArg,

    /// Min reconnect backoff in ms (kube streams)
    #[arg(long, default_value_t = 200)]
    pub reconnect_min_ms: u64,

    /// Max reconnect backoff in ms (kube streams)
    #[arg(long, default_value_t = 5000)]
    pub reconnect_max_ms: u64,
}
