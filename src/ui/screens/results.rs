//! Results screen
use crate::{
    sty,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{
        Constraint::{Length, Min},
        Layout, Rect,
    },
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, sync::mpsc::SyncSender};

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

    fn handle_events(&mut self, key: KeyEvent, _state: &mut Self::State) {
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
        let [text_a, footer_a] = Layout::vertical([Min(0), Length(1)]).areas(area);

        // body text
        let b = Block::new()
            .borders(Borders::TOP)
            .style(sty!(accent))
            .title("Results & Analysis".bold())
            .padding(Padding::horizontal(1));
        let p = Paragraph::new("results go here")
            .style(sty!(root))
            .wrap(Wrap { trim: false })
            .block(b);

        p.render(text_a, buf);

        // footer
        Line::from("Use ⭡/⭣ to scroll or 'q' to go back.")
            .style(sty!(accent))
            .centered()
            .render(footer_a, buf);
    }
}
