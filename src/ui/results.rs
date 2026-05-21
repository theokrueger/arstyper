//! Results screen
use crate::{
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, rc::Rc, sync::mpsc::SyncSender};

/// Results state
pub struct ResultsState {}

impl ArstyperWidgetState for ResultsState {
    fn new() -> io::Result<Self> {
        Ok(Self {})
    }
}

/// Results screen
pub struct Results {
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Results {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { styles: s, tx: tx })
    }

    fn handle_events(&mut self, _key: KeyEvent, _state: &mut Self::State) {}
}

impl StatefulWidgetRef for Results {
    type State = ResultsState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Paragraph::new("results go here")
            .style(self.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(self.styles.accent)
                    .title("Results and Analysis".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
