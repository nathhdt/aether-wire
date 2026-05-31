//! TUI progress bar module

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct ProgressBar {
    ratio: f64,
    style: Style,
    percent_style: Style,
    show_percentage: bool,
}

impl ProgressBar {
    pub fn new(ratio: f64) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            style: Style::default(),
            percent_style: Style::default(),
            show_percentage: true,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn percent_style(mut self, style: Style) -> Self {
        self.percent_style = style;
        self
    }

    fn build_bar(&self, width: usize) -> String {
        const PARTIALS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

        let total_units = (self.ratio * width as f64 * 8.0).round() as usize;

        let full = total_units / 8;
        let partial = total_units % 8;

        let mut bar = String::new();

        bar.push_str(&"█".repeat(full));

        if partial > 0 {
            bar.push_str(PARTIALS[partial]);
        }

        let used = full + usize::from(partial > 0);

        if used < width {
            bar.push_str(&" ".repeat(width - used));
        }

        bar
    }
}

impl Widget for ProgressBar {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let percent_width = if self.show_percentage { 5 } else { 0 };

        let bar_width = area.width.saturating_sub(percent_width) as usize;

        let bar_text = self.build_bar(bar_width);

        let mut spans = vec![Span::styled(bar_text, self.style)];

        if self.show_percentage {
            let percent_text = format!(" {:>3}%", (self.ratio * 100.0).round() as u8);
            spans.push(Span::styled(percent_text, self.percent_style));
        }

        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
