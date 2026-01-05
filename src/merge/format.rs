use crate::types::{ColorBy, ColorMode, LogEvent, OutputConfig, OutputMode};
use owo_colors::OwoColorize;
use std::hash::{Hash, Hasher};
use std::io::IsTerminal;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const LABEL_COL_WIDTH: usize = 36;

pub fn format_event(ev: &LogEvent, out: &OutputConfig) -> String {
    match out.mode {
        OutputMode::Human => format_human(ev, out),
        OutputMode::Json => format_json(ev),
    }
}

fn format_json(ev: &LogEvent) -> String {
    let ts = format_ts(&ev.ts);

    let obj = serde_json::json!({
        "ts": ts,
        "namespace": ev.namespace,
        "pod": ev.pod,
        "container": ev.container,
        "message": ev.message,
    });

    obj.to_string()
}

fn format_human(ev: &LogEvent, out: &OutputConfig) -> String {
    let ts = format_ts(&ev.ts);

    let label_plain = format!("{}/{}", ev.pod, ev.container);

    let label_padded = pad_label(&label_plain, LABEL_COL_WIDTH);

    let label_final = if should_color(out) {
        colorize_label(label_padded, out.color_by, &ev.pod, &ev.container)
    } else {
        label_padded
    };

    format!("{ts} {label_final} │ {}", ev.message)
}

fn format_ts(ts: &OffsetDateTime) -> String {
    ts.format(&Rfc3339).unwrap_or_else(|_| ts.to_string())
}

fn pad_label(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{s:<width$}", width = width)
    }
}

fn should_color(out: &OutputConfig) -> bool {
    match out.color {
        ColorMode::Never => false,
        ColorMode::Always => true,
        ColorMode::Auto => std::io::stdout().is_terminal(),
    }
}

fn colorize_label(padded: String, color_by: ColorBy, pod: &str, container: &str) -> String {
    let key = match color_by {
        ColorBy::Pod => pod,
        ColorBy::Container => container,
    };

    let idx = stable_color_index(key);

    match idx {
        0 => padded.bright_blue().to_string(),
        1 => padded.bright_green().to_string(),
        2 => padded.bright_magenta().to_string(),
        3 => padded.bright_cyan().to_string(),
        4 => padded.bright_yellow().to_string(),
        5 => padded.bright_red().to_string(),
        6 => padded.blue().to_string(),
        7 => padded.green().to_string(),
        8 => padded.magenta().to_string(),
        9 => padded.cyan().to_string(),
        _ => padded.yellow().to_string(),
    }
}

fn stable_color_index(s: &str) -> usize {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    (h.finish() as usize) % 11
}
