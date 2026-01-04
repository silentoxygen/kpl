use serde::Serialize;
use std::fmt;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PodKey {
    pub namespace: String,
    pub name: String,
    pub uid: String,
}

impl fmt::Display for PodKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} ({})", self.namespace, self.name, self.uid)
    }
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

#[derive(Debug, Clone, Copy)]
pub enum TimestampSource {
    Local,
    Kube, // reserved for later when we enable kube timestamps parsing
}

impl TimestampSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimestampSource::Local => "local",
            TimestampSource::Kube => "kube",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorBy {
    Pod,
    Container,
}

impl ColorBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorBy::Pod => "pod",
            ColorBy::Container => "container",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub color: bool,
    pub color_by: ColorBy,
    pub timestamps: bool,
}

#[derive(Debug, Clone)]
pub struct KubeLogOpts {
    pub containers: Vec<String>, // if empty => all containers
    pub since_seconds: Option<i64>,
    pub tail_lines: Option<i64>,
    pub timestamps_source: TimestampSource,
    pub reconnect_min_ms: u64,
    pub reconnect_max_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DevOpts {
    pub rate_ms: u64,
    pub lines: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RuntimeOpts {
    pub buffer: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEvent {
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub message: String,
}
