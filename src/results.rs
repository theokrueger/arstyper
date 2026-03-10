//! Typing test struct
use crate::ui::{Screen, Styles, UiRequest};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{cmp::min, sync::mpsc::SyncSender, time::Instant};

/// Typing test results
pub struct Results {
    styles: Styles,
    /// Message to the UI to be performed on next tick. Didn't feel like using an actual message system lmao
    tx: SyncSender<UiRequest>,
}

impl Results {
    /// Create a new emtpy test, which must be initialised before use :D
    pub fn new(s: Styles, tx: SyncSender<UiRequest>) -> Self {
        Self { styles: s, tx: tx }
    }

    /// Render the results
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("res")
            .style(self.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(self.styles.accent)
                    .title("Results".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
