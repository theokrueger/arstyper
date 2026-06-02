use crate::ui::UiRequest;
use ratatui::{
    buffer::Buffer, crossterm::event::KeyEvent, layout::Rect, widgets::StatefulWidgetRef,
};
use std::{io, sync::mpsc::SyncSender};

/// Renderable Stateful Widget for Arstyper
pub trait ArstyperWidget: StatefulWidgetRef + Sized {
    /// Handle crossterm events for this widget
    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State);

    /// Create a new, empty version of this widget
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self>;
}

pub trait ArstyperWidgetState: Sized {
    fn new() -> io::Result<Self>;
}

pub trait ArstyperOverlay: ArstyperWidget {
    fn render_ref_overlay(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}
