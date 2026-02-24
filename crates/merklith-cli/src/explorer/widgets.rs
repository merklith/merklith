//! Custom widgets for the TUI explorer.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// A simple sparkline widget for showing trends
pub struct MiniSparkline {
    data: Vec<u64>,
    max: u64,
}

impl MiniSparkline {
    pub fn new(data: Vec<u64>) -> Self {
        let max = data.iter().copied().max().unwrap_or(1);
        Self { data, max }
    }
}

impl Widget for MiniSparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        
        for (i, &value) in self.data.iter().take(area.width as usize).enumerate() {
            let idx = ((value as f64 / self.max as f64) * (chars.len() - 1) as f64) as usize;
            let char = chars[idx.min(chars.len() - 1)];
            
            if i < area.width as usize {
                let x = area.x + i as u16;
                let y = area.y;
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(char);
                }
            }
        }
    }
}

/// Status indicator widget
pub struct StatusIndicator {
    connected: bool,
}

impl StatusIndicator {
    pub fn new(connected: bool) -> Self {
        Self { connected }
    }
}

impl Widget for StatusIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (symbol, color) = if self.connected {
            ("●", Color::Green)
        } else {
            ("●", Color::Red)
        };
        
        let text = Text::from(Span::styled(symbol, Style::default().fg(color)));
        Paragraph::new(text).render(area, buf);
    }
}

/// Loading spinner widget
pub struct Spinner {
    frame: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self { frame: 0 }
    }
    
    pub fn next(mut self) -> Self {
        self.frame = (self.frame + 1) % 8;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
        let char = frames[self.frame % frames.len()];
        
        if let Some(cell) = buf.cell_mut((area.x, area.y)) {
            cell.set_char(char);
        }
    }
}