use serde::Serialize;
use time::OffsetDateTime;

/// Pod identity (use UID to avoid confusing replaced pods).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PodKey {
    pub namespace: String,
    pub name: String,
    pub uid: String,
}

/// Stream identity = Pod + Container.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StreamKey {
    pub pod: PodKey,
    pub container: String,
}

/// Event produced by streamers and consumed by merger/output.
/// For v1: timestamp is "ingest time" at the client.
#[derive(Clone, Debug, Serialize)]
pub struct LogEvent {
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub message: String,
}
