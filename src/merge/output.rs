use std::io::{self, Write};

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::errors::AppResult;
use crate::merge::format::{format_human, maybe_colorize};
use crate::types::{LogEvent, OutputConfig, OutputMode};

pub async fn run_merger(
    mut rx: mpsc::Receiver<LogEvent>,
    output: OutputConfig,
    shutdown: CancellationToken,
) -> AppResult<()> {
    // stdout lock is !Send; keep this on the current task
    let stdout = io::stdout();
    let mut out = stdout.lock();

    loop {
        let next = tokio::select! {
            _ = shutdown.cancelled() => {
                // graceful shutdown requested
                break;
            }
            ev = rx.recv() => ev
        };

        let Some(ev) = next else {
            // all senders dropped -> we're done
            break;
        };

        let res: io::Result<()> = match output.mode {
            OutputMode::Human => {
                let line = format_human(&ev, &output);
                let line = maybe_colorize(line, &ev, &output);
                writeln!(out, "{line}")
            }
            OutputMode::Json => {
                serde_json::to_writer(&mut out, &ev).map_err(io::Error::other)?;
                writeln!(out)
            }
        };

        if let Err(e) = res {
            if e.kind() == io::ErrorKind::BrokenPipe {
                // piping to head etc -> treat as normal exit
                return Ok(());
            }
            return Err(e.into());
        }
    }

    Ok(())
}
