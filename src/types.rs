use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PodKey {
    pub namespace: String,
    pub name: String,
    pub uid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamKey {
    pub pod: PodKey,
    pub container: String,
}

#[derive(Debug, Clone)]
pub enum PodCommand {
    StartPod {
        pod: PodKey,
        containers: Vec<String>,
    },
    StopPod {
        pod: PodKey,
    },
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub ts: OffsetDateTime,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorBy {
    Pod,
    Container,
}

impl fmt::Display for ColorBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorBy::Pod => write!(f, "pod"),
            ColorBy::Container => write!(f, "container"),
        }
    }
}

/// Whether to emit ANSI colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::Always => write!(f, "always"),
            ColorMode::Never => write!(f, "never"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub color_by: ColorBy,
    pub color: ColorMode,
    pub no_color: bool,
}
