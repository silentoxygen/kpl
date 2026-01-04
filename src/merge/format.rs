use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::types::{ColorBy, LogEvent, OutputConfig};

/// Format one LogEvent in "human" mode.
/// Example:
/// 2025-12-28T18:27:53.513Z dev-pod-1/app | message
pub fn format_human(ev: &LogEvent, cfg: &OutputConfig) -> String {
    let mut s = String::new();

    if cfg.timestamps {
        // RFC3339 for display (matches JSON serialization)
        let ts = ev
            .ts
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        s.push_str(&ts);
        s.push(' ');
    }

    // prefix: pod/container
    s.push_str(&ev.pod);
    s.push('/');
    s.push_str(&ev.container);
    s.push_str(" | ");

    s.push_str(&ev.message);
    s
}

/// Wrap a formatted line in ANSI color, if enabled.
pub fn maybe_colorize(line: String, ev: &LogEvent, cfg: &OutputConfig) -> String {
    if !cfg.color {
        return line;
    }

    // Choose stable "color id" from hash.
    let key = match cfg.color_by {
        ColorBy::Pod => StableKey::Pod(&ev.pod),
        ColorBy::Container => StableKey::Container(&ev.pod, &ev.container),
    };

    let code = ansi_color_code(stable_hash(key));
    format!("\x1b[{}m{}\x1b[0m", code, line)
}

enum StableKey<'a> {
    Pod(&'a str),
    Container(&'a str, &'a str),
}

fn stable_hash(key: StableKey<'_>) -> u64 {
    let mut h = DefaultHasher::new();
    match key {
        StableKey::Pod(p) => p.hash(&mut h),
        StableKey::Container(p, c) => {
            p.hash(&mut h);
            c.hash(&mut h);
        }
    }
    h.finish()
}

fn ansi_color_code(hash: u64) -> u8 {
    // Pick from readable 8-bit ANSI "bright-ish" colors (avoid black/gray).
    // 31..36 basic colors, plus 91..96 bright variants.
    const CODES: [u8; 12] = [31, 32, 33, 34, 35, 36, 91, 92, 93, 94, 95, 96];
    let idx = (hash as usize) % CODES.len();
    CODES[idx]
}
