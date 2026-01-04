use std::io::{self, Write};

use tokio::sync::mpsc;

use crate::config::{OutputConfig, OutputMode};
use crate::merge::format::LineFormatter;
use crate::types::LogEvent;

pub async fn run_merger(mut rx: mpsc::Receiver<LogEvent>, output: OutputConfig) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let formatter = LineFormatter::new(output.human);

    while let Some(ev) = rx.recv().await {
        let write_result: io::Result<()> = match output.mode {
            OutputMode::Human => {
                let line = formatter.format_human(&ev);
                out.write_all(line.as_bytes())?;
                out.write_all(b"\n")?;
                Ok(())
            }
            OutputMode::Json => {
                serde_json::to_writer(&mut out, &ev).map_err(io::Error::other)?;
                out.write_all(b"\n")?;
                Ok(())
            }
        };

        if let Err(e) = write_result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }
    }

    Ok(())
}
