//! Statistics screen
use crate::{
    globs,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::UiRequest,
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, sync::mpsc::SyncSender};

/// Results state
pub struct StatsState {}

impl ArstyperWidgetState for StatsState {
    fn new() -> io::Result<Self> {
        Ok(Self {})
    }
}

/// Stats screen
pub struct Stats {
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Stats {
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { tx: tx })
    }

    fn handle_events(&mut self, _key: KeyEvent, _state: &mut Self::State) {}
}

impl StatefulWidgetRef for Stats {
    type State = StatsState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Paragraph::new("stats go here")
            .style(globs::sty().root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(globs::sty().accent)
                    .title("Statistics".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
