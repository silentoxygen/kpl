use clap::{Parser, ValueEnum};

use crate::types::{ColorBy, ColorMode};

#[derive(Debug, Parser)]
#[command(
    name = "kube-podlog",
    version,
    about = "Fast multi-pod Kubernetes log tailer"
)]
pub struct Cli {
    /// Namespace
    #[arg(short = 'n', long = "namespace", default_value = "default")]
    pub namespace: String,

    /// Label selector (e.g. app=web,tier=frontend)
    #[arg(short = 'l', long = "selector")]
    pub selector: String,

    /// Emit NDJSON log events
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,

    /// Color mode: auto (tty only), always, never
    #[arg(long = "color", value_enum, default_value_t = ColorModeArg::Auto)]
    pub color: ColorModeArg,

    /// Color by: pod or container
    #[arg(long = "color-by", value_enum, default_value_t = ColorByArg::Pod)]
    pub color_by: ColorByArg,

    /// Disable colors (overrides --color)
    #[arg(long = "no-color", default_value_t = false)]
    pub no_color: bool,

    /// Dev mode: simulate pods without a cluster
    #[arg(long = "dev", default_value_t = false)]
    pub dev: bool,

    /// Dev: milliseconds between lines
    #[arg(long = "dev-rate-ms", default_value_t = 500)]
    pub dev_rate_ms: u64,

    /// Dev: lines per container per phase
    #[arg(long = "dev-lines", default_value_t = 10)]
    pub dev_lines: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ColorModeArg {
    Auto,
    Always,
    Never,
}

impl From<ColorModeArg> for ColorMode {
    fn from(v: ColorModeArg) -> Self {
        match v {
            ColorModeArg::Auto => ColorMode::Auto,
            ColorModeArg::Always => ColorMode::Always,
            ColorModeArg::Never => ColorMode::Never,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ColorByArg {
    Pod,
    Container,
}

impl From<ColorByArg> for ColorBy {
    fn from(v: ColorByArg) -> Self {
        match v {
            ColorByArg::Pod => ColorBy::Pod,
            ColorByArg::Container => ColorBy::Container,
        }
    }
}
