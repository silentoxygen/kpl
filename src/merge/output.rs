use crate::merge::format::format_event;
use crate::types::{LogEvent, OutputConfig};
use std::io::{self, Write};
use tokio::sync::mpsc;

pub async fn run_merger(mut rx: mpsc::Receiver<LogEvent>, output: OutputConfig) -> io::Result<()> {
    while let Some(ev) = rx.recv().await {
        let line = format_event(&ev, &output);

        let mut out = io::stdout().lock();

        if let Err(e) = writeln!(out, "{line}") {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }

        let _ = out.flush();
    }

    Ok(())
}
