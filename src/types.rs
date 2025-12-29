use serde::Serialize;
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

#[derive(Debug, Clone, Serialize)]
pub struct LogEvent {
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub message: String,
}
