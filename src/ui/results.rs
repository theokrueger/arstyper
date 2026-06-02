//! Results screen
use crate::{
    consts::{self, Styles},
    sty,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
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
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Results {
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State) {
        match key.code {
            KeyCode::Enter => {
                self.tx.send(UiRequest::NewTest).unwrap();
                self.tx
                    .send(UiRequest::ChangeScreen(Screen::TestScreen))
                    .unwrap();
            }
            _ => {}
        }
    }
}

impl StatefulWidgetRef for Results {
    type State = ResultsState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Paragraph::new("results go here")
            .style(sty!(root))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(sty!(accent))
                    .title("Results and Analysis".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
