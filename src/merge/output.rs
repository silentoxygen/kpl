use tokio::sync::mpsc;

use crate::config::OutputMode;
use crate::types::LogEvent;

pub async fn run_merger(mut rx: mpsc::Receiver<LogEvent>, mode: OutputMode) {
    while let Some(ev) = rx.recv().await {
        match mode {
            OutputMode::Human => {
                println!(
                    "{} {} {} {}",
                    ev.ts
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    ev.pod,
                    ev.container,
                    ev.message
                );
            }
            OutputMode::Json => {
                if let Ok(line) = serde_json::to_string(&ev) {
                    println!("{line}");
                }
            }
        }
    }
}
