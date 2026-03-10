//! Test results struct
use crate::ui::{Styles, UiRequest};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{rc::Rc, sync::mpsc::SyncSender};

/// Typing test results
pub struct Results {
    styles: Rc<Styles>,
    /// Message to the UI to be performed on next tick. Didn't feel like using an actual message system lmao
    tx: SyncSender<UiRequest>,
}

impl Results {
    /// Create a new emtpy test, which must be initialised before use :D
    pub fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self {
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
