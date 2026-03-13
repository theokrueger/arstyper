//! Results screen
use crate::{
    traits::ArstyperScreen,
    ui::{Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{rc::Rc, sync::mpsc::SyncSender};

/// Results screen
pub struct Results {
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

impl ArstyperScreen for Results {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self {
        Self { styles: s, tx: tx }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("results go here")
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

    fn handle_events(&mut self, key: KeyEvent) {}
}
