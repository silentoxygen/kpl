use anstyle::{AnsiColor, Effects, Style};

use crate::config::{ColorBy, HumanFormat};
use crate::types::LogEvent;

pub struct LineFormatter {
    human: HumanFormat,
    pod_width: usize,
    container_width: usize,
}

impl LineFormatter {
    pub fn new(human: HumanFormat) -> Self {
        Self {
            human,
            pod_width: 20,
            container_width: 12,
        }
    }

    pub fn format_human(&self, ev: &LogEvent) -> String {
        let (pod_s, container_s) = if self.human.color {
            let style = self.style_for(ev);
            (paint(&ev.pod, style), paint(&ev.container, style))
        } else {
            (ev.pod.clone(), ev.container.clone())
        };

        if self.human.timestamps {
            let ts = ev
                .ts
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "<bad-ts>".to_string());

            format!(
                "{}  {:<pw$}  {:<cw$}  {}",
                ts,
                pod_s,
                container_s,
                ev.message,
                pw = self.pod_width,
                cw = self.container_width
            )
        } else {
            format!(
                "{:<pw$}  {:<cw$}  {}",
                pod_s,
                container_s,
                ev.message,
                pw = self.pod_width,
                cw = self.container_width
            )
        }
    }

    fn style_for(&self, ev: &LogEvent) -> Style {
        let key = match self.human.color_by {
            ColorBy::Pod => ev.pod.as_str(),
            ColorBy::Container => ev.container.as_str(),
        };

        let idx = stable_color_index(key);

        let color = match idx {
            0 => AnsiColor::Green,
            1 => AnsiColor::Cyan,
            2 => AnsiColor::Yellow,
            3 => AnsiColor::Magenta,
            4 => AnsiColor::Blue,
            _ => AnsiColor::Red,
        };

        Style::new()
            .fg_color(Some(color.into()))
            .effects(Effects::BOLD)
    }
}

fn paint(s: &str, style: Style) -> String {
    format!("{}{}{}", style.render(), s, style.render_reset())
}

fn stable_color_index(s: &str) -> usize {
    // Small stable hash (FNV-1a style), avoids extra deps.
    let mut h: u64 = 1469598103934665603;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    (h as usize) % 6
}
