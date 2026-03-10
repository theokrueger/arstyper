//! "About this program" screen
use crate::{
    traits::ArstyperScreen,
    ui::{Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{rc::Rc, sync::mpsc::SyncSender};

/// About screen
pub struct About {
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

impl ArstyperScreen for About {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self {
        Self { styles: s, tx: tx }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("help text")
            .style(self.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(self.styles.accent)
                    .title("About".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
